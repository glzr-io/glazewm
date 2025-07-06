use std::cell::RefCell;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::HWND,
  UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{
      EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
      EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
      EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
      EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, OBJID_WINDOW,
      WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    },
  },
};

use crate::{platform_impl::Installable, WindowEvent, WindowEventType};

thread_local! {
  static WINDOW_EVENTS: RefCell<Option<WindowEventHandles>> = const { RefCell::new(None) };
}

/// Holds objects related to the window event hook on the event thread.
struct WindowEventHandles {
  event_tx: mpsc::UnboundedSender<WindowEvent>,
  hook_handles: Vec<HWINEVENTHOOK>,
}

/// Window event hook to be used in the main program.
///
/// Receives window events from the event loop.
#[derive(Debug)]
pub struct WindowEventHook {
  rx: mpsc::UnboundedReceiver<WindowEvent>,
}

impl WindowEventHook {
  /// Creates an instance of `WindowEventHook`.
  #[allow(clippy::type_complexity)]
  pub(crate) fn new(
    event_types: &'static [WindowEventType],
  ) -> (
    Self,
    Installable<
      impl FnOnce() -> anyhow::Result<()> + Send + 'static,
      impl FnOnce() -> anyhow::Result<()> + Send + 'static,
    >,
  ) {
    let (tx, rx) = mpsc::unbounded_channel::<WindowEvent>();

    let install = move || {
      // Collect all the event IDs from the provided event types.
      let mut ids: Vec<u32> = event_types
        .iter()
        .map(crate::events::WindowEventType::id)
        .collect();
      ids.sort_unstable();

      // Create ranges of consecutive event IDs.
      // Results in the minimum needed number of hooks.
      let mut iter = ids.iter();
      let mut ranges = vec![];
      while let Some(id) = iter.next() {
        let mut max = *id;
        for next in iter.by_ref() {
          if *next == id + 1 {
            max = *next;
          }
        }

        ranges.push((*id, max));
      }

      // Create a hook for each range of event IDs.
      let handles: Vec<HWINEVENTHOOK> = ranges
        .iter()
        .map(|range| {
          let hook_handle: anyhow::Result<HWINEVENTHOOK> =
            Self::hook_win_event(range.0, range.1);
          hook_handle
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

      let event_handles = WindowEventHandles {
        event_tx: tx,
        hook_handles: handles,
      };

      WINDOW_EVENTS.with(|w| {
        w.replace(Some(event_handles));
      });

      Ok(())
    };

    let stop = move || {
      tracing::info!("Stopping window event hook");
      WINDOW_EVENTS.with(|handles| {
        let handles = if let Some(handles) = handles.replace(None) {
          handles.hook_handles
        } else {
          return;
        };
        for hook_handle in handles {
          unsafe { UnhookWinEvent(hook_handle) };
        }
      });

      Ok(())
    };

    let installer = Installable {
      installer: install,
      stop,
    };
    let win_event_hook = Self { rx };

    (win_event_hook, installer)
  }

  /// Creates a window hook for the specified event range.
  fn hook_win_event(
    event_min: u32,
    event_max: u32,
  ) -> Result<HWINEVENTHOOK> {
    let hook_handle = unsafe {
      SetWinEventHook(
        event_min,
        event_max,
        None,
        Some(window_event_hook_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
      )
    };

    if hook_handle.is_invalid() {
      Err(anyhow::anyhow!("Failed to set window event hook."))
    } else {
      Ok(hook_handle)
    }
  }

  /// Stops the window event hook and unhooks from all window events.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn stop(&self) -> anyhow::Result<()> {
    Ok(())
  }

  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    self.rx.recv().await
  }
}

/// Callback passed to `SetWinEventHook` to handle window events.
///
/// This function is called on selected window events, and forwards them
/// through an MPSC channel for the WM to process.
extern "system" fn window_event_hook_proc(
  _hook: HWINEVENTHOOK,
  event_type: u32,
  handle: HWND,
  id_object: i32,
  id_child: i32,
  _event_thread: u32,
  _event_time: u32,
) {
  let is_window_event =
    id_object == OBJID_WINDOW.0 && id_child == 0 && handle != HWND(0);

  // Check whether the event is associated with a window object instead
  // of a UI control.
  if !is_window_event {
    return;
  }

  let window = crate::NativeWindow::new(handle.0);

  let window_event = match event_type {
    EVENT_OBJECT_DESTROY => WindowEvent::WindowDestroyed(window),
    EVENT_SYSTEM_FOREGROUND => WindowEvent::WindowFocused(window),
    EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED => {
      WindowEvent::WindowHidden(window)
    }
    EVENT_OBJECT_LOCATIONCHANGE => {
      WindowEvent::WindowLocationChanged(window)
    }
    EVENT_SYSTEM_MINIMIZESTART => WindowEvent::WindowMinimized(window),
    EVENT_SYSTEM_MINIMIZEEND => WindowEvent::WindowMinimizeEnded(window),
    EVENT_SYSTEM_MOVESIZEEND => {
      WindowEvent::WindowMovedOrResizedEnd(window)
    }
    EVENT_SYSTEM_MOVESIZESTART => {
      WindowEvent::WindowMovedOrResizedStart(window)
    }
    EVENT_OBJECT_SHOW | EVENT_OBJECT_UNCLOAKED => {
      WindowEvent::WindowShown(window)
    }
    EVENT_OBJECT_NAMECHANGE => WindowEvent::WindowTitleChanged(window),
    _ => return,
  };

  WINDOW_EVENTS.with(|hooks| {
    if let Some(ref hooks) = *hooks.borrow() {
      if let Err(err) = hooks.event_tx.send(window_event) {
        warn!("Failed to send platform event '{}'.", err);
      }
    }
  });
}
