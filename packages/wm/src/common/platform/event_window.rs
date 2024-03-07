use std::{cell::OnceCell, thread};

use anyhow::{bail, Result};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, warn};
use windows::Win32::{
  Foundation::HWND,
  UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{
      DispatchMessageW, GetMessageW, TranslateMessage,
      EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS,
      EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE,
      EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
      EVENT_SYSTEM_MOVESIZEEND, MSG, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
      WINEVENT_SKIPOWNPROCESS,
    },
  },
};

use super::{NativeWindow, PlatformEvent};

thread_local! {
  static HOOK_EVENT_TX: OnceCell<UnboundedSender<PlatformEvent>> = OnceCell::new();
}

extern "system" fn event_hook_proc(
  hook: HWINEVENTHOOK,
  event: u32,
  hwnd: HWND,
  id_object: i32,
  id_child: i32,
  event_thread: u32,
  event_time: u32,
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
        EVENT_OBJECT_SHOW => PlatformEvent::WindowShown(window),
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

pub struct EventWindow;

impl EventWindow {
  pub fn new() -> Self {
    let event_window_thread = thread::spawn(|| unsafe {
      let event_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_OBJECT_DESTROY,
        None,
        Some(event_hook_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
      );

      if event_hook.is_invalid() {
        bail!("`SetWinEventHook` failed.");
      }

      let mut msg = MSG::default();
      while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }

      let true = UnhookWinEvent(event_hook).as_bool() else {
        bail!("`UnhookWinEvent` failed.");
      };

      Ok(())
    });

    match event_window_thread.join() {
      Err(err) => error!("join th: {:?}", err),
      _ => (),
    }

    Self
  }
}
