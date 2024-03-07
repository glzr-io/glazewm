use std::{
  cell::OnceCell,
  thread::{self, JoinHandle},
};

use anyhow::{bail, Result};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::warn;
use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
      Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
        PostQuitMessage, RegisterClassW, TranslateMessage, CS_HREDRAW,
        CS_VREDRAW, CW_USEDEFAULT, DBT_DEVNODES_CHANGED,
        EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
        EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
        EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
        EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND, MSG,
        OBJID_WINDOW, SPI_ICONVERTICALSPACING, SPI_SETWORKAREA,
        WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_DESTROY,
        WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_SETTINGCHANGE, WNDCLASSW,
        WS_OVERLAPPEDWINDOW,
      },
    },
  },
};

use super::{NativeWindow, PlatformEvent};

thread_local! {
  static HOOK_EVENT_TX: OnceCell<UnboundedSender<PlatformEvent>> = OnceCell::new();
}

/// Callback passed to `SetWinEventHook` to handle window events.
///
/// This function is called on selected window events, and forwards them
/// through an MPSC channel for the WM to process.
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
        warn!("Failed to send platform event '{}'.", err);
      }
    }
  });
}

/// Window procedure for the event window.
///
/// This function handles messages for the event window, and forwards
/// display change events through an MPSC channel for the WM to process.
pub extern "system" fn event_window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  HOOK_EVENT_TX.with(|event_tx| {
    if let Some(event_tx) = event_tx.get() {
      return match msg {
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
          handle_display_change_msg(msg, wparam, event_tx)
        }
        WM_DESTROY => {
          unsafe { PostQuitMessage(0) };
          LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
      };
    }

    LRESULT(0)
  })
}

/// Handle display change messages and emit the corresponding event.
fn handle_display_change_msg(
  msg: u32,
  wparam: WPARAM,
  event_tx: &UnboundedSender<PlatformEvent>,
) -> LRESULT {
  let should_emit_event = match msg {
    WM_SETTINGCHANGE => {
      wparam.0 as u32 == SPI_SETWORKAREA.0
        || wparam.0 as u32 == SPI_ICONVERTICALSPACING.0
    }
    WM_DEVICECHANGE => wparam.0 as u32 == DBT_DEVNODES_CHANGED,
    _ => true,
  };

  if should_emit_event {
    let event = PlatformEvent::DisplaySettingsChanged;
    if let Err(err) = event_tx.send(event) {
      warn!("Failed to send platform event '{}'.", err);
    }
  }

  LRESULT(0)
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

  /// Create several window event hooks via `SetWinEventHook`.
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

  /// Create a window hook for the specified event range.
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

  /// Create the event window and its message loop.
  unsafe fn create_message_loop(
    mut abort_rx: oneshot::Receiver<()>,
  ) -> Result<()> {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("EventWindow"),
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: Some(event_window_proc),
      ..Default::default()
    };

    RegisterClassW(&wnd_class);

    let hwnd = CreateWindowExW(
      Default::default(),
      w!("EventWindow"),
      w!("EventWindow"),
      WS_OVERLAPPEDWINDOW,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      None,
      None,
      wnd_class.hInstance,
      None,
    );

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

  /// Destroy the event window and stop the message loop.
  pub fn destroy(&mut self) {
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
