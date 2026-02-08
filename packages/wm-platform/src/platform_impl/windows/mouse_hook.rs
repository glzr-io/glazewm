use std::sync::Arc;

use tracing::warn;
use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
  },
  Foundation::{HWND, LPARAM, LRESULT, POINT},
  UI::{
    Input::{
      GetRawInputData,
      KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON},
      RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
      RAWINPUTHEADER, RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT,
      RIM_TYPEMOUSE,
    },
    WindowsAndMessaging::{
      GetCursorPos, RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
      RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP, WM_INPUT,
    },
  },
};

use super::FOREGROUND_INPUT_IDENTIFIER;
use crate::{
  mouse_listener::MouseEventType, platform_event::MouseEventNotification,
  Dispatcher, DispatcherExtWindows, MouseButton, Point, WindowId,
};

/// Windows-specific mouse event notification.
#[derive(Clone, Debug)]
pub struct MouseEventNotificationInner {
  event_type: MouseEventType,
  position: Point,
  pressed_buttons: Vec<MouseButton>,
}

impl MouseEventNotificationInner {
  /// Returns the type of mouse event.
  #[must_use]
  pub fn event_type(&self) -> MouseEventType {
    self.event_type.clone()
  }

  /// Returns the cursor position at the time of the event.
  #[must_use]
  pub fn position(&self) -> Point {
    self.position.clone()
  }

  /// Returns the mouse buttons that were pressed at the time of the
  /// event.
  #[must_use]
  pub fn pressed_buttons(&self) -> Vec<MouseButton> {
    self.pressed_buttons.clone()
  }
}

/// A callback invoked for every mouse notification received.
type HookCallback = dyn Fn(MouseEventNotification) + Send + Sync + 'static;

/// Windows-specific mouse hook that listens for configured mouse events.
pub struct MouseHook {
  callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl MouseHook {
  /// Creates a new mouse hook with the specified enabled mouse event
  /// kinds and callback.
  ///
  /// The callback is executed for every received mouse notification
  /// whose event type is in `enabled_events`.
  pub fn new<F>(
    enabled_events: &[MouseEventType],
    callback: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self>
  where
    F: Fn(MouseEventNotification) + Send + Sync + 'static,
  {
    let callback = Arc::new(callback);
    let enabled_events =
      Arc::from(enabled_events.to_vec().into_boxed_slice());

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, msg, _wparam, lparam| {
        if msg != WM_INPUT {
          return None;
        }

        if let Err(err) =
          Self::handle_wm_input(lparam, &enabled_events, &*callback)
        {
          warn!("Failed to handle WM_INPUT message: {}", err);
        }

        Some(LRESULT(0))
      },
    ))?;

    let handle = dispatcher.message_window_handle();

    // Register raw input devices on the event loop thread so that
    // `WM_INPUT` messages are delivered to the message window.
    dispatcher.dispatch_sync(move || {
      if let Err(err) = Self::register_raw_input(handle, true) {
        warn!("Failed to register raw input devices: {}", err);
      }
    })?;

    Ok(Self {
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
    })
  }

  /// Terminates the hook.
  pub fn terminate(&mut self) -> crate::Result<()> {
    if let Some(id) = self.callback_id.take() {
      self.dispatcher.deregister_wndproc_callback(id)?;

      // Deregister raw input on the event loop thread.
      let handle = self.dispatcher.message_window_handle();
      self.dispatcher.dispatch_sync(|| {
        if let Err(err) = Self::register_raw_input(handle, false) {
          warn!("Failed to deregister raw input devices: {}", err);
        }
      })?;
    }

    Ok(())
  }

  /// Processes a single `WM_INPUT` message, extracting the raw input
  /// data and invoking the user callback if the event type is enabled.
  fn handle_wm_input(
    lparam: LPARAM,
    enabled_events: &[MouseEventType],
    callback: &HookCallback,
  ) -> crate::Result<()> {
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
    // process are filtered, since `NativeWindow::focus` simulates mouse
    // input.
    #[allow(clippy::cast_possible_truncation)]
    if res_size == 0
      || raw_input_size == u32::MAX
      || raw_input.header.dwType != RIM_TYPEMOUSE.0
      || unsafe { raw_input.data.mouse.ulExtraInformation } as u32
        == FOREGROUND_INPUT_IDENTIFIER
    {
      return Ok(());
    }

    let button_flags =
      unsafe { raw_input.data.mouse.Anonymous.Anonymous.usButtonFlags };

    let event_type = Self::event_type_from_flags(button_flags);

    // Only invoke the callback if the event type is enabled.
    if !enabled_events.contains(&event_type) {
      return Ok(());
    }

    // TODO: Avoid querying for cursor position and pressed buttons.
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&raw mut point) }?;

    let position = Point {
      x: point.x,
      y: point.y,
    };

    let pressed_buttons = Self::current_pressed_buttons();

    let notification =
      MouseEventNotification(MouseEventNotificationInner {
        event_type,
        position,
        pressed_buttons,
      });

    callback(notification);

    Ok(())
  }

  /// Maps raw input button flags to a `MouseEventType`.
  ///
  /// Returns `Move` when no button state change is present.
  fn event_type_from_flags(flags: u16) -> MouseEventType {
    let flags_u32 = u32::from(flags);

    if flags_u32 & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
      MouseEventType::LeftClickDown
    } else if flags_u32 & RI_MOUSE_LEFT_BUTTON_UP != 0 {
      MouseEventType::LeftClickUp
    } else if flags_u32 & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
      MouseEventType::RightClickDown
    } else if flags_u32 & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
      MouseEventType::RightClickUp
    } else {
      MouseEventType::Move
    }
  }

  /// Queries the current pressed mouse buttons via `GetAsyncKeyState`.
  fn current_pressed_buttons() -> Vec<MouseButton> {
    let mut pressed = Vec::new();

    // High-order bit set indicates the key is currently down.
    if (unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) } as u16 & 0x8000)
      != 0
    {
      pressed.push(MouseButton::Left);
    }

    if (unsafe { GetAsyncKeyState(VK_RBUTTON.0.into()) } as u16 & 0x8000)
      != 0
    {
      pressed.push(MouseButton::Right);
    }

    pressed
  }

  /// Enables or disables the mouse hook.
  pub fn enable(&mut self, enabled: bool) -> crate::Result<()> {
    if self.callback_id.is_some() {
      let handle = self.dispatcher.message_window_handle();
      self.dispatcher.dispatch_sync(move || {
        Self::register_raw_input(handle, enabled)
      })??;
    }
    Ok(())
  }

  /// Registers or deregisters the raw input device for mouse events with
  /// `RIDEV_INPUTSINK` or `RIDEV_REMOVE`.
  fn register_raw_input(
    target_handle: WindowId,
    enabled: bool,
  ) -> crate::Result<()> {
    let mode_flag = if enabled {
      RIDEV_INPUTSINK
    } else {
      RIDEV_REMOVE
    };

    let target_hwnd = if enabled {
      HWND(target_handle.0)
    } else {
      HWND::default()
    };

    let rid = RAWINPUTDEVICE {
      usUsagePage: HID_USAGE_PAGE_GENERIC,
      usUsage: HID_USAGE_GENERIC_MOUSE,
      dwFlags: mode_flag,
      hwndTarget: target_hwnd,
    };

    unsafe {
      #[allow(clippy::cast_possible_truncation)]
      RegisterRawInputDevices(
        &[rid],
        std::mem::size_of::<RAWINPUTDEVICE>() as u32,
      )
    }?;

    Ok(())
  }
}

impl Drop for MouseHook {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      warn!("Failed to terminate mouse hook: {}", err);
    }
  }
}
