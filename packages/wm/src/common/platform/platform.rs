use std::sync::Arc;

use anyhow::bail;
use tokio::sync::{oneshot, Mutex};
use tracing::warn;
use windows::core::w;
use windows::Win32::UI::WindowsAndMessaging::{
  CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW,
  RegisterClassW, SetCursorPos, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
  CW_USEDEFAULT, MSG, WNDCLASSW, WNDPROC, WS_OVERLAPPEDWINDOW,
};
use windows::Win32::UI::{
  HiDpi::{
    SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
  },
  WindowsAndMessaging::{GetDesktopWindow, GetForegroundWindow},
};

use crate::common::Rect;
use crate::user_config::UserConfig;

use super::{
  native_monitor, native_window, EventListener, NativeMonitor,
  NativeWindow, SingleInstance, WindowHandle,
};

pub type WindowProcedure = WNDPROC;

pub struct Platform;

impl Platform {
  pub fn foreground_window() -> NativeWindow {
    let handle = unsafe { GetForegroundWindow() };
    NativeWindow::new(handle)
  }

  pub fn desktop_window() -> NativeWindow {
    let handle = unsafe { GetDesktopWindow() };
    NativeWindow::new(handle)
  }

  /// Monitors sorted from left-to-right, top-to-bottom.
  ///
  /// Note that this also ensures that the `NativeMonitor` instances have
  /// valid position values.
  pub fn sorted_monitors() -> anyhow::Result<Vec<NativeMonitor>> {
    let monitors = native_monitor::available_monitors()?;

    // Create a tuple of monitor and its rect.
    let mut monitors_with_rect = monitors
      .into_iter()
      .map(|monitor| {
        let rect = monitor.rect()?.clone();
        anyhow::Ok((monitor, rect))
      })
      .try_collect::<Vec<_>>()?;

    // Sort monitors from left-to-right, top-to-bottom.
    monitors_with_rect.sort_by(|(_, rect_a), (_, rect_b)| {
      if rect_a.x() == rect_b.x() {
        rect_a.y().cmp(&rect_b.y())
      } else {
        rect_a.x().cmp(&rect_b.x())
      }
    });

    // Convert back to a regular vector of monitors.
    Ok(
      monitors_with_rect
        .into_iter()
        .map(|(monitor, _)| monitor)
        .collect(),
    )
  }

  pub fn nearest_monitor(window: &NativeWindow) -> NativeMonitor {
    native_monitor::nearest_monitor(window.handle)
  }

  pub fn manageable_windows() -> anyhow::Result<Vec<NativeWindow>> {
    Ok(
      native_window::available_windows()?
        .into_iter()
        .filter(|w| w.is_manageable())
        .collect(),
    )
  }

  pub async fn new_event_listener(
    config: &Arc<Mutex<UserConfig>>,
  ) -> anyhow::Result<EventListener> {
    EventListener::start(config).await
  }

  pub fn new_single_instance() -> anyhow::Result<SingleInstance> {
    SingleInstance::new()
  }

  pub fn set_dpi_awareness() -> anyhow::Result<()> {
    unsafe {
      SetProcessDpiAwarenessContext(
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
      )
    }?;

    Ok(())
  }

  pub fn set_cursor_pos(x: i32, y: i32) -> anyhow::Result<()> {
    unsafe {
      SetCursorPos(x, y)?;
    };

    Ok(())
  }

  /// Creates a message window and starts a message loop.
  pub unsafe fn create_message_loop(
    mut abort_rx: oneshot::Receiver<()>,
    window_procedure: WindowProcedure,
  ) -> anyhow::Result<WindowHandle> {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("MessageWindow"),
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: window_procedure,
      ..Default::default()
    };

    RegisterClassW(&wnd_class);

    let handle = CreateWindowExW(
      Default::default(),
      w!("MessageWindow"),
      w!("MessageWindow"),
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

    if handle.0 == 0 {
      bail!("Creation of message window failed.");
    }

    let mut msg = MSG::default();

    loop {
      // Check whether the abort signal has been received.
      if abort_rx.try_recv().is_ok() {
        if let Err(err) = DestroyWindow(handle) {
          warn!("Failed to destroy message window '{}'.", err);
        }
        break;
      }

      if GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      } else {
        break;
      }
    }

    Ok(handle)
  }
}
