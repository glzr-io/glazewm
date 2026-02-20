use std::{
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
  },
  Foundation::{HWND, POINT},
  UI::{
    Input::{
      GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
      RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RIDEV_REMOVE,
      RID_INPUT, RIM_TYPEMOUSE,
    },
    WindowsAndMessaging::{
      GetCursorPos, RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
      RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP, WM_INPUT,
    },
  },
};

use super::FOREGROUND_INPUT_IDENTIFIER;
use crate::{
  mouse_listener::MouseEventKind,
  platform_event::{MouseButton, MouseEvent, PressedButtons},
  Dispatcher, DispatcherExtWindows, Point,
};

/// Data shared with the window procedure callback.
struct CallbackData {
  event_tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>,

  /// Pressed button state tracked from events.
  pressed: PressedButtons,

  /// Timestamp of the last emitted `Move` event for throttling.
  last_move_emission: Option<Instant>,
}

/// Windows-specific implementation of [`MouseListener`].
pub(crate) struct MouseListener {
  callback_id: Option<usize>,
  callback_data: Arc<Mutex<CallbackData>>,
  dispatcher: Dispatcher,
  event_tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>,
}

impl MouseListener {
  /// Windows-specific implementation of [`MouseListener::new`].
  pub(crate) fn new(
    enabled_events: &[MouseEventKind],
    event_tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let callback_data = Arc::new(Mutex::new(CallbackData {
      event_tx: event_tx.clone(),
      pressed: PressedButtons::default(),
      last_move_emission: None,
    }));

    let callback_id = Self::register_callback(
      enabled_events,
      Arc::clone(&callback_data),
      dispatcher,
    )?;

    Ok(Self {
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
      event_tx,
      callback_data,
    })
  }

  /// Registers a window procedure callback for `WM_INPUT` and enables raw
  /// input.
  ///
  /// Returns the ID of the created callback.
  fn register_callback(
    enabled_events: &[MouseEventKind],
    callback_data: Arc<Mutex<CallbackData>>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<usize> {
    let enabled_events: Arc<[MouseEventKind]> =
      Arc::from(enabled_events.to_vec().into_boxed_slice());

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, msg, _wparam, lparam| {
        if msg != WM_INPUT {
          return None;
        }

        let mut callback_data = callback_data
          .lock()
          .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Err(err) =
          Self::handle_wm_input(lparam, enabled_events, &mut callback_data)
        {
          tracing::warn!("Failed to handle WM_INPUT message: {}", err);
        }

        Some(0)
      },
    ))?;

    // Register raw input devices, which will then deliver `WM_INPUT`
    // messages to the event loop's message window.
    let handle = dispatcher.message_window_handle();
    dispatcher
      .dispatch_sync(move || Self::enable_raw_input(handle, true))??;

    Ok(callback_id)
  }

  /// Windows-specific implementation of [`MouseListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    self.enable(false)?;

    if let Some(id) = self.callback_id.take() {
      self.dispatcher.deregister_wndproc_callback(id)?;
    }

    Ok(())
  }

  /// Windows-specific implementation of [`MouseListener::enable`].
  pub(crate) fn enable(&mut self, enabled: bool) -> crate::Result<()> {
    if self.callback_id.is_some() {
      let handle = self.dispatcher.message_window_handle();
      self.dispatcher.dispatch_sync(move || {
        Self::enable_raw_input(handle, enabled)
      })??;
    }

    Ok(())
  }

  /// Windows-specific implementation of
  /// [`MouseListener::set_enabled_events`].
  pub(crate) fn set_enabled_events(
    &mut self,
    enabled_events: Vec<MouseEventKind>,
  ) -> crate::Result<()> {
    let _ = self.terminate();

    let callback_id = Self::register_callback(
      &enabled_events,
      Arc::clone(&self.callback_data),
      &self.dispatcher,
    )?;

    self.callback_id = Some(callback_id);

    Ok(())
  }

  /// Processes a `WM_INPUT` message, extracting raw input data and
  /// sending the appropriate [`MouseEvent`] on the channel.
  fn handle_wm_input(
    lparam: isize,
    enabled_events: &[MouseEventKind],
    callback_data: &mut CallbackData,
  ) -> crate::Result<()> {
    let mut raw_input: RAWINPUT = unsafe { std::mem::zeroed() };
    #[allow(clippy::cast_possible_truncation)]
    let mut raw_input_size = std::mem::size_of::<RAWINPUT>() as u32;

    let res_size = unsafe {
      #[allow(clippy::cast_possible_truncation)]
      GetRawInputData(
        HRAWINPUT(lparam),
        RID_INPUT,
        Some(std::ptr::from_mut(&mut raw_input).cast()),
        &raw mut raw_input_size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
      )
    };

    // Ignore if input is invalid or not a mouse event. Inputs from our own
    // process are also ignored, since `NativeWindow::focus` simulates
    // mouse input.
    #[allow(clippy::cast_possible_truncation)]
    if res_size == 0
      || raw_input_size == u32::MAX
      || raw_input.header.dwType != RIM_TYPEMOUSE.0
      || unsafe { raw_input.data.mouse.ulExtraInformation } as u32
        == FOREGROUND_INPUT_IDENTIFIER
    {
      return Ok(());
    }

    // Map button flags to a `MouseEventKind`.
    let event_kind = {
      let button_flags = u32::from(unsafe {
        raw_input.data.mouse.Anonymous.Anonymous.usButtonFlags
      });

      // Button flags indicate a transition in mouse button state.
      // Ref: https://learn.microsoft.com/en-us/windows/win32/api/ntddmou/ns-ntddmou-mouse_input_data#members
      if button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
        MouseEventKind::LeftButtonDown
      } else if button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
        MouseEventKind::LeftButtonUp
      } else if button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
        MouseEventKind::RightButtonDown
      } else if button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
        MouseEventKind::RightButtonUp
      } else {
        MouseEventKind::Move
      }
    };

    if !enabled_events.contains(&event_kind) {
      return Ok(());
    }

    // Throttle mouse move events so that there's a minimum of 50ms between
    // each emission. State change events (button down/up) always get
    // emitted.
    let should_emit = match event_kind {
      MouseEventKind::Move => {
        callback_data.last_move_emission.is_none_or(|timestamp| {
          timestamp.elapsed() >= Duration::from_millis(50)
        })
      }
      _ => true,
    };

    if !should_emit {
      return Ok(());
    }

    callback_data.pressed.update(event_kind);

    let mouse_event = match event_kind {
      MouseEventKind::LeftButtonDown => MouseEvent::ButtonDown {
        position: Self::cursor_pos()?,
        button: MouseButton::Left,
        pressed_buttons: callback_data.pressed,
      },
      MouseEventKind::LeftButtonUp => MouseEvent::ButtonUp {
        position: Self::cursor_pos()?,
        button: MouseButton::Left,
        pressed_buttons: callback_data.pressed,
      },
      MouseEventKind::RightButtonDown => MouseEvent::ButtonDown {
        position: Self::cursor_pos()?,
        button: MouseButton::Right,
        pressed_buttons: callback_data.pressed,
      },
      MouseEventKind::RightButtonUp => MouseEvent::ButtonUp {
        position: Self::cursor_pos()?,
        button: MouseButton::Right,
        pressed_buttons: callback_data.pressed,
      },
      MouseEventKind::Move => MouseEvent::Move {
        position: Self::cursor_pos()?,
        pressed_buttons: callback_data.pressed,
        window_below_cursor: None,
      },
    };

    let _ = callback_data.event_tx.send(mouse_event);

    if event_kind == MouseEventKind::Move {
      callback_data.last_move_emission = Some(Instant::now());
    }

    Ok(())
  }

  /// Gets the current cursor position.
  fn cursor_pos() -> crate::Result<Point> {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&raw mut point) }?;
    Ok(Point {
      x: point.x,
      y: point.y,
    })
  }

  /// Registers or deregisters the raw input device for mouse events.
  fn enable_raw_input(
    target_handle: isize,
    enabled: bool,
  ) -> crate::Result<()> {
    let mode_flag = if enabled {
      RIDEV_INPUTSINK
    } else {
      RIDEV_REMOVE
    };

    let target_hwnd = if enabled {
      HWND(target_handle)
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
    }
    .map_err(crate::Error::from)
  }
}

impl Drop for MouseListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      tracing::warn!("Failed to terminate mouse listener: {}", err);
    }
  }
}
