use std::{
  cell::OnceCell,
  thread::{self, JoinHandle},
};

use anyhow::{bail, Result};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::warn;
use windows::Win32::{
  Foundation::HWND,
  UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{
      DispatchMessageW, GetMessageW, TranslateMessage,
      EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
      EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
      EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
      EVENT_SYSTEM_MOVESIZEEND, MSG, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
      WINEVENT_SKIPOWNPROCESS,
    },
  },
};

use super::{NativeWindow, PlatformEvent};

thread_local! {
  // static HOOK_EVENT_TX: OnceCell<Arc<UnboundedSender<PlatformEvent>>> = OnceCell::new();
  static HOOK_EVENT_TX: OnceCell<UnboundedSender<PlatformEvent>> = OnceCell::new();
}

extern "system" fn event_hook_proc(
  _hook: HWINEVENTHOOK,
  event: u32,
  hwnd: HWND,
  id_object: i32,
  id_child: i32,
  _event_thread: u32,
  _event_time: u32,
) {
  HOOK_EVENT_TX.with(|event_tx| {
    if let Some(event_tx) = event_tx.get() {
      let is_window_event =
        id_object == OBJID_WINDOW.0 && id_child == 0 && hwnd != HWND(0);

      // Check whether the event is associated with a window object instead
      // of a UI control.
      if !is_window_event {
        return;
      }

      let window = NativeWindow::new(hwnd);

      let platform_event = match event {
        EVENT_OBJECT_DESTROY => PlatformEvent::WindowDestroyed(window),
        EVENT_SYSTEM_FOREGROUND => PlatformEvent::WindowFocused(window),
        EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED => {
          PlatformEvent::WindowHidden(window)
        }
        EVENT_OBJECT_LOCATIONCHANGE => {
          PlatformEvent::WindowLocationChanged(window)
        }
        EVENT_SYSTEM_MINIMIZESTART => {
          PlatformEvent::WindowMinimized(window)
        }
        EVENT_SYSTEM_MINIMIZEEND => {
          PlatformEvent::WindowMinimizeEnded(window)
        }
        EVENT_SYSTEM_MOVESIZEEND => {
          PlatformEvent::WindowMovedOrResized(window)
        }
        EVENT_OBJECT_SHOW | EVENT_OBJECT_UNCLOAKED => {
          PlatformEvent::WindowShown(window)
        }
        EVENT_OBJECT_NAMECHANGE => {
          PlatformEvent::WindowTitleChanged(window)
        }
        _ => return,
      };

      if let Err(err) = event_tx.send(platform_event) {
        warn!("Failed to send platform event '{}'", err);
      }
    }
  });
}

#[derive(Debug)]
pub struct EventWindow {
  abort_tx: Option<oneshot::Sender<()>>,
  window_thread: Option<JoinHandle<Result<()>>>,
}

impl EventWindow {
  pub fn new(event_tx: UnboundedSender<PlatformEvent>) -> Self {
    let (abort_tx, abort_rx) = oneshot::channel();

    let window_thread = thread::spawn(|| unsafe {
      // Initialize the `HOOK_EVENT_TX` thread-local static.
      HOOK_EVENT_TX.with(|cell| cell.set(event_tx)).unwrap();

      let hook_handles = Self::hook_win_events()?;

      Self::create_message_loop(abort_rx)?;

      // Unhook from all window events.
      for hook_handle in hook_handles {
        if let false = UnhookWinEvent(hook_handle).as_bool() {
          bail!("`UnhookWinEvent` failed.");
        }
      }

      Ok(())
    });

    Self {
      abort_tx: Some(abort_tx),
      window_thread: Some(window_thread),
    }
  }

  fn hook_win_events() -> Result<Vec<HWINEVENTHOOK>> {
    let event_ranges = [
      (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE),
      (EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE),
      (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
      (EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND),
      (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
      (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE),
      (EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED),
    ];

    // Create separate hooks for each event range. This is more performant
    // than creating a single hook for all events and filtering them.
    event_ranges
      .iter()
      .try_fold(Vec::new(), |mut handles, event_range| {
        let hook_handle =
          unsafe { Self::hook_win_event(event_range.0, event_range.1) }?;
        handles.push(hook_handle);
        Ok(handles)
      })
  }

  unsafe fn hook_win_event(
    event_min: u32,
    event_max: u32,
  ) -> Result<HWINEVENTHOOK> {
    let hook_handle = SetWinEventHook(
      event_min,
      event_max,
      None,
      Some(event_hook_proc),
      0,
      0,
      WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
    );

    if hook_handle.is_invalid() {
      bail!("`SetWinEventHook` failed.");
    }

    Ok(hook_handle)
  }

  unsafe fn create_message_loop(
    mut abort_rx: oneshot::Receiver<()>,
  ) -> Result<()> {
    let mut msg = MSG::default();

    loop {
      // Check whether the abort signal has been received.
      if abort_rx.try_recv().is_ok() {
        break;
      }

      if GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      } else {
        break;
      }
    }

    Ok(())
  }

  pub fn destroy(&mut self) {
    // Send a signal to the spawned thread to stop the message loop.
    // if self.abort_tx.send(()).is_err() {
    if let Some(abort_tx) = self.abort_tx.take() {
      if abort_tx.send(()).is_err() {
        warn!("Failed to send abort signal to the event window thread.");
      }
    }

    // Wait for the spawned thread to finish.
    if let Some(window_thread) = self.window_thread.take() {
      if let Err(err) = window_thread.join() {
        warn!("Failed to join event window thread '{:?}'.", err);
      }
    }
  }
}

impl Drop for EventWindow {
  fn drop(&mut self) {
    self.destroy();
  }
}
