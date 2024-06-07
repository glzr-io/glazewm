use std::{
  ops::BitAnd,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, OnceLock,
  },
  thread::{self, JoinHandle},
};

use tokio::sync::mpsc;
use tracing::{error, info, warn};
use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC, MOUSE_MOVE_ABSOLUTE,
    MOUSE_MOVE_RELATIVE,
  },
  Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
  UI::{
    Input::{
      GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
      RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK,
      RID_DEVICE_INFO_TYPE, RID_INPUT, RIM_TYPEMOUSE,
    },
    WindowsAndMessaging::{
      DefWindowProcW, DestroyWindow, GetCursorPos, PostQuitMessage,
      DBT_DEVNODES_CHANGED, RI_MOUSE_LEFT_BUTTON_DOWN,
      RI_MOUSE_LEFT_BUTTON_UP, RI_MOUSE_RIGHT_BUTTON_DOWN,
      RI_MOUSE_RIGHT_BUTTON_UP, SPI_ICONVERTICALSPACING, SPI_SETWORKAREA,
      WM_DESTROY, WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_INPUT,
      WM_POWERBROADCAST, WM_SETTINGCHANGE,
    },
  },
};

use crate::{common::Point, user_config::KeybindingConfig};

use super::{
  KeyboardHook, MouseHook, MouseMoveEvent, Platform, PlatformEvent,
  WinEventHook,
};

/// Global instance of sender for platform events.
///
/// For use with window procedure.
static PLATFORM_EVENT_TX: OnceLock<mpsc::UnboundedSender<PlatformEvent>> =
  OnceLock::new();

/// Whether mouse hook is currently enabled.
///
/// For use with window procedure.
static IS_MOUSE_HOOK_ENABLED: AtomicBool = AtomicBool::new(false);

/// Whether left-click is currently pressed.
///
/// For use with window procedure.
static IS_L_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Whether right-click is currently pressed.
///
/// For use with window procedure.
static IS_R_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Timestamp of the last event emission.
///
/// For use with window procedure.
static LAST_MOUSE_EVENT_TIME: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct EventWindow {
  keyboard_hook: Arc<KeyboardHook>,
  mouse_hook: Arc<MouseHook>,
  win_event_hook: Arc<WinEventHook>,
  window_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl EventWindow {
  /// Creates an instance of `EventWindow`. Spawns a hidden window and
  /// emits platform events.
  ///
  /// Uses global state (e.g. `PLATFORM_EVENT_TX`) and should thus only
  /// ever be instantiated once in the application's lifetime.
  pub fn new(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
    keybindings: Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) -> anyhow::Result<Self> {
    let keyboard_hook = KeyboardHook::new(keybindings, event_tx.clone())?;
    let mouse_hook = MouseHook::new(event_tx.clone())?;
    let win_event_hook = WinEventHook::new(event_tx.clone())?;

    let keyboard_hook_clone = keyboard_hook.clone();
    let win_event_hook_clone = win_event_hook.clone();

    let window_thread = thread::spawn(move || {
      // Add the sender for platform events to global state.
      PLATFORM_EVENT_TX.set(event_tx.clone()).map_err(|_| {
        anyhow::anyhow!("Platform event sender already set.")
      })?;

      IS_MOUSE_HOOK_ENABLED.store(enable_mouse_events, Ordering::Relaxed);

      // Start hooks for listening to platform events.
      keyboard_hook_clone.start()?;
      win_event_hook_clone.start()?;

      // Create a hidden window with a message loop on the current thread.
      let handle =
        Platform::create_message_window(Some(event_window_proc))?;

      let rid = RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC,
        usUsage: HID_USAGE_GENERIC_MOUSE,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: HWND(handle),
      };

      // Register our window to receive mouse events.
      unsafe {
        RegisterRawInputDevices(
          &[rid],
          std::mem::size_of::<RAWINPUTDEVICE>() as u32,
        )
      }?;

      Platform::run_message_loop();
      unsafe { DestroyWindow(HWND(handle)) }?;

      Ok(())
    });

    Ok(Self {
      keyboard_hook,
      mouse_hook,
      win_event_hook,
      window_thread: Some(window_thread),
    })
  }

  pub fn update(
    &mut self,
    keybindings: Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) {
    self.keyboard_hook.update(keybindings);
    match enable_mouse_events {
      true => _ = self.mouse_hook.start(),
      false => self.mouse_hook.stop(),
    }
  }

  /// Destroys the event window and stops the message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down event window.");

    self.keyboard_hook.stop();
    self.mouse_hook.stop();
    self.win_event_hook.stop();

    // Wait for the spawned thread to finish.
    if let Some(window_thread) = self.window_thread.take() {
      Platform::kill_message_loop(&window_thread)?;

      window_thread
        .join()
        .map_err(|_| anyhow::anyhow!("Thread join failed."))??;
    }

    Ok(())
  }
}

impl Drop for EventWindow {
  fn drop(&mut self) {
    if let Err(err) = self.destroy() {
      error!("Failed to gracefully shut down event window: {}", err);
    }
  }
}

/// Window procedure for the event window.
///
/// Handles messages for the event window, and forwards display change
/// events through an MPSC channel for the WM to process.
pub extern "system" fn event_window_proc(
  handle: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if let Some(event_tx) = PLATFORM_EVENT_TX.get() {
    return match message {
      WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
        handle_display_change_msg(message, wparam, event_tx)
      }
      // WM_INPUT if IS_MOUSE_HOOK_ENABLED.load(Ordering::Relaxed) => {
      WM_INPUT => handle_input_msg(wparam, lparam, event_tx),
      _ => unsafe { DefWindowProcW(handle, message, wparam, lparam) },
    };
  }

  LRESULT(0)
}

/// Handles display change messages and emits the corresponding platform
/// event through an MPSC channel.
fn handle_display_change_msg(
  message: u32,
  wparam: WPARAM,
  event_tx: &mpsc::UnboundedSender<PlatformEvent>,
) -> LRESULT {
  let should_emit_event = match message {
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

/// Handles raw input messages for mouse events and emits the corresponding
/// platform event through an MPSC channel.
fn handle_input_msg(
  _wparam: WPARAM,
  lparam: LPARAM,
  event_tx: &mpsc::UnboundedSender<PlatformEvent>,
) -> LRESULT {
  // let mut raw_input: RAWINPUT = unsafe { std::mem::zeroed() };
  let mut raw_input: RAWINPUT = RAWINPUT::default();
  let mut raw_input_size = std::mem::size_of::<RAWINPUT>() as u32;

  let res_size = unsafe {
    GetRawInputData(
      HRAWINPUT(lparam.0),
      RID_INPUT,
      Some(&mut raw_input as *mut _ as _),
      &mut raw_input_size,
      std::mem::size_of::<RAWINPUTHEADER>() as u32,
    )
  };

  // Ignore if data is invalid or not a mouse event.
  if res_size == 0
    || raw_input_size == u32::MAX
    || raw_input.header.dwType != RIM_TYPEMOUSE.0
  {
    return LRESULT(0);
  }

  let mouse_input = unsafe { raw_input.data.mouse };
  let state_flags = mouse_input.usFlags;
  let button_flags =
    unsafe { mouse_input.Anonymous.Anonymous.usButtonFlags };

  if has_flags(button_flags, RI_MOUSE_LEFT_BUTTON_DOWN) {
    IS_L_MOUSE_DOWN.store(true, Ordering::Relaxed);
  }

  if has_flags(button_flags, RI_MOUSE_LEFT_BUTTON_UP) {
    IS_L_MOUSE_DOWN.store(false, Ordering::Relaxed);
  }

  if has_flags(button_flags, RI_MOUSE_RIGHT_BUTTON_DOWN) {
    IS_R_MOUSE_DOWN.store(true, Ordering::Relaxed);
  }

  if has_flags(button_flags, RI_MOUSE_RIGHT_BUTTON_UP) {
    IS_R_MOUSE_DOWN.store(false, Ordering::Relaxed);
  }

  if has_flags(state_flags, MOUSE_MOVE_RELATIVE)
    || has_flags(state_flags, MOUSE_MOVE_ABSOLUTE)
  {
    let is_mouse_down = IS_L_MOUSE_DOWN.load(Ordering::Relaxed)
      || IS_R_MOUSE_DOWN.load(Ordering::Relaxed);

    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut point) };

    let event = MouseMoveEvent {
      point: Point {
        x: point.x,
        y: point.y,
      },
      is_mouse_down,
    };

    if let Err(err) = event_tx.send(PlatformEvent::MouseMove(event)) {
      warn!("Failed to send platform event '{}'.", err);
    }

    // LAST_MOUSE_EVENT_TIME.store(mouse_event.time, Ordering::Relaxed);
  }

  LRESULT(0)
}

/// Checks whether `short` contains all the bits of `mask`.
#[inline]
fn has_flags(short: u16, mask: u32) -> bool {
  short & mask as u16 == mask as u16
}
