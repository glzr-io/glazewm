use std::{
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    OnceLock,
  },
  time::SystemTime,
};

use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
  },
  Foundation::{HWND, LPARAM, POINT, WPARAM},
  UI::{
    Input::{
      GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
      RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT,
      RIM_TYPEMOUSE,
    },
    WindowsAndMessaging::{
      GetCursorPos, RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
      RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
    },
  },
};
use wm_common::Point;

use crate::{
  platform_impl::{Installable, FOREGROUND_INPUT_IDENTIFIER},
  MouseEvent, MouseMoveEvent,
};

static MOUSE_EVENT_TX: OnceLock<
  tokio::sync::mpsc::UnboundedSender<crate::MouseEvent>,
> = const { OnceLock::new() };

/// Whether mouse hook is currently enabled.
///
/// For use with window procedure.
static ENABLE_MOUSE_EVENTS: AtomicBool = AtomicBool::new(false);

/// Whether left-click is currently pressed.
///
/// For use with window procedure.
static IS_L_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Whether right-click is currently pressed.
///
/// For use with window procedure.
static IS_R_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Timestamp of the last mouse event emission.
///
/// For use with window procedure.
static LAST_MOUSE_EVENT_TIME: AtomicU64 = AtomicU64::new(0);

pub struct MouseHook {
  event_rx: tokio::sync::mpsc::UnboundedReceiver<crate::MouseEvent>,
}

impl MouseHook {
  /// Creates a new [`MouseHook`] that listens for mouse events on the
  /// specified window. Returns a tuple containing the [`MouseHook`]
  /// instance and a function to install the hook on the message thread.
  #[allow(clippy::type_complexity)]
  pub fn new(
    window_handle: crate::WindowHandle,
  ) -> anyhow::Result<(
    Self,
    Installable<
      impl FnOnce() -> anyhow::Result<()>,
      impl FnOnce() -> anyhow::Result<()>,
    >,
  )> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let install = move || {
      let rid = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC,
        usUsage: HID_USAGE_GENERIC_MOUSE,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: HWND(window_handle),
      };

      // Register our window to receive mouse events.
      unsafe {
        #[allow(clippy::cast_possible_truncation)]
        RegisterRawInputDevices(
          &[rid],
          std::mem::size_of::<RAWINPUTDEVICE>() as u32,
        )
      }?;

      MOUSE_EVENT_TX.set(tx).map_err(|_| {
        anyhow::anyhow!("Mouse event transmitter already set.")
      })?;

      Ok(())
    };

    let stop = || {
      tracing::info!("Stopping mouse hook");
      Ok(())
    };

    let install = Installable {
      installer: install,
      stop,
    };

    Ok((Self { event_rx: rx }, install))
  }

  pub fn update(enable_events: bool) {
    ENABLE_MOUSE_EVENTS.store(enable_events, Ordering::Relaxed);
  }

  pub async fn next_event(&mut self) -> Option<crate::MouseEvent> {
    self.event_rx.recv().await
  }

  pub fn try_next_event(&mut self) -> Option<crate::MouseEvent> {
    self.event_rx.try_recv().ok()
  }

  /// Handles raw input messages for mouse events and emits the
  /// corresponding platform event through an MPSC channel.
  pub fn handle_mouse_input(
    _wparam: WPARAM,
    lparam: LPARAM,
  ) -> anyhow::Result<()> {
    let event_tx = MOUSE_EVENT_TX.get().ok_or_else(|| {
      anyhow::anyhow!("Mouse event transmitter not set.")
    })?;

    let mut raw_input: RAWINPUT = unsafe { std::mem::zeroed() };
    #[allow(clippy::cast_possible_truncation)]
    let mut raw_input_size = std::mem::size_of::<RAWINPUT>() as u32;

    let res_size = unsafe {
      #[allow(clippy::cast_possible_truncation)]
      GetRawInputData(
        HRAWINPUT(lparam.0),
        RID_INPUT,
        Some(std::ptr::from_mut(&mut raw_input).cast()),
        &raw mut raw_input_size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
      )
    };

    // Ignore if data is invalid or not a mouse event. Inputs from our own
    // process are ignored, which would cause issues since
    // `NativeWindow::set_foreground` simulates a mouse input.
    if res_size == 0
      || raw_input_size == u32::MAX
      || raw_input.header.dwType != RIM_TYPEMOUSE.0
      || unsafe { raw_input.data.mouse.ulExtraInformation }
        == FOREGROUND_INPUT_IDENTIFIER
    {
      return Ok(());
    }

    let mouse_input = unsafe { raw_input.data.mouse };
    let button_flags =
      unsafe { mouse_input.Anonymous.Anonymous.usButtonFlags };

    let has_state_change = match button_flags {
      flags if Self::has_mouse_flag(flags, RI_MOUSE_LEFT_BUTTON_DOWN) => {
        IS_L_MOUSE_DOWN.store(true, Ordering::Relaxed);
        true
      }
      flags if Self::has_mouse_flag(flags, RI_MOUSE_LEFT_BUTTON_UP) => {
        IS_L_MOUSE_DOWN.store(false, Ordering::Relaxed);
        true
      }
      flags if Self::has_mouse_flag(flags, RI_MOUSE_RIGHT_BUTTON_DOWN) => {
        IS_R_MOUSE_DOWN.store(true, Ordering::Relaxed);
        true
      }
      flags if Self::has_mouse_flag(flags, RI_MOUSE_RIGHT_BUTTON_UP) => {
        IS_R_MOUSE_DOWN.store(false, Ordering::Relaxed);
        true
      }
      _ => false,
    };

    #[allow(clippy::cast_possible_truncation)]
    let event_time = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .map(|dur| dur.as_millis() as u64)?;

    let last_event_time = LAST_MOUSE_EVENT_TIME.load(Ordering::Relaxed);

    // Throttle events so that there's a minimum of 50ms between each
    // emission. Always emit if there's a state change.
    if !has_state_change && event_time - last_event_time < 50 {
      return Ok(());
    }

    let is_mouse_down = IS_L_MOUSE_DOWN.load(Ordering::Relaxed)
      || IS_R_MOUSE_DOWN.load(Ordering::Relaxed);

    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&raw mut point) }?;

    event_tx.send(MouseEvent::MouseMove(MouseMoveEvent {
      point: Point {
        x: point.x,
        y: point.y,
      },
      is_mouse_down,
    }))?;

    LAST_MOUSE_EVENT_TIME.store(event_time, Ordering::Relaxed);

    Ok(())
  }

  /// Checks whether `state` contains all the bits of `mask`.
  #[inline]
  fn has_mouse_flag(state: u16, mask: u32) -> bool {
    u32::from(state) & mask == mask
  }
}
