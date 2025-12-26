use std::{
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, OnceLock,
  },
  thread::{self, JoinHandle},
  time::SystemTime,
};

use tokio::sync::mpsc;
use tracing::{info, warn};
use windows::Win32::{
  Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
  },
  Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
  UI::{
    Input::{
      GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
      RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT,
      RIM_TYPEMOUSE,
    },
    WindowsAndMessaging::{
      DefWindowProcW, DestroyWindow, GetCursorPos, DBT_DEVNODES_CHANGED,
      PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
      RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
      RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
      SPI_ICONVERTICALSPACING, SPI_SETWORKAREA, WM_DEVICECHANGE,
      WM_DISPLAYCHANGE, WM_INPUT, WM_POWERBROADCAST, WM_SETTINGCHANGE,
    },
  },
};
use wm_common::{KeybindingConfig, Point};

// Import the MouseHook here
use crate::mouse_hook::MouseHook;

use super::{
  KeyboardHook, MouseMoveEvent, Platform, PlatformEvent, WindowEventHook,
  FOREGROUND_INPUT_IDENTIFIER,
};

/// Global instance of sender for platform events.
static PLATFORM_EVENT_TX: OnceLock<mpsc::UnboundedSender<PlatformEvent>> =
  OnceLock::new();

/// Whether mouse hook is currently enabled.
static ENABLE_MOUSE_EVENTS: AtomicBool = AtomicBool::new(false);

/// Whether the system is currently sleeping/hibernating.
static IS_SYSTEM_SUSPENDED: AtomicBool = AtomicBool::new(false);

/// Whether left-click is currently pressed.
static IS_L_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Whether right-click is currently pressed.
static IS_R_MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

/// Timestamp of the last mouse event emission.
static LAST_MOUSE_EVENT_TIME: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct EventWindow {
  keyboard_hook: Arc<KeyboardHook>,
  // mouse_hook field removed to prevent dead code warning
  window_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl EventWindow {
  pub fn new(
    event_tx: &mpsc::UnboundedSender<PlatformEvent>,
    keybindings: &Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) -> anyhow::Result<Self> {
    let keyboard_hook = KeyboardHook::new(keybindings, event_tx.clone())?;
    let window_event_hook = WindowEventHook::new(event_tx.clone())?;
    
    // Initialize MouseHook
    let mouse_hook = MouseHook::new(event_tx.clone())?;

    let keyboard_hook_clone = keyboard_hook.clone();
    
    // Clone MouseHook for the thread
    let mouse_hook_clone = mouse_hook.clone();

    // Add the sender for platform events to global state.
    PLATFORM_EVENT_TX.set(event_tx.clone()).map_err(|_| {
      anyhow::anyhow!("Platform event sender already set.")
    })?;

    ENABLE_MOUSE_EVENTS.store(enable_mouse_events, Ordering::Relaxed);

    let window_thread = thread::spawn(move || {
      // Start hooks for listening to platform events.
      keyboard_hook_clone.start()?;
      window_event_hook.start()?;
      
      // Start MouseHook
      mouse_hook_clone.start()?;

      // Create a hidden window with a message loop on the current thread.
      // We must define 'handle' here because it is used below for RegisterRawInputDevices and DestroyWindow
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
        #[allow(clippy::cast_possible_truncation)]
        RegisterRawInputDevices(
          &[rid],
          std::mem::size_of::<RAWINPUTDEVICE>() as u32,
        )
      }?;

      Platform::run_message_loop();

      // Clean-up on message loop exit.
      unsafe { DestroyWindow(HWND(handle)) }?;
      keyboard_hook_clone.stop()?;
      window_event_hook.stop()?;
      
      // Stop MouseHook
      mouse_hook_clone.stop()?;

      Ok(())
    });

    Ok(Self {
      keyboard_hook,
      // mouse_hook is active in the thread, but we don't need to store it in the struct
      window_thread: Some(window_thread),
    })
  }

  pub fn update(
    &mut self,
    keybindings: &Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) {
    self.keyboard_hook.update(keybindings);
    ENABLE_MOUSE_EVENTS.store(enable_mouse_events, Ordering::Relaxed);
  }

  /// Destroys the event window and stops the message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down event window.");

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
      warn!("Failed to gracefully shut down event window: {}", err);
    }
  }
}

/// Window procedure for the event window.
pub extern "system" fn event_window_proc(
  handle: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if let Some(event_tx) = PLATFORM_EVENT_TX.get() {
    return match message {
      WM_POWERBROADCAST => {
        #[allow(clippy::cast_possible_truncation)]
        match wparam.0 as u32 {
          PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
            IS_SYSTEM_SUSPENDED.store(false, Ordering::Relaxed);
          }
          PBT_APMSUSPEND => {
            IS_SYSTEM_SUSPENDED.store(true, Ordering::Relaxed);
          }
          _ => {}
        }

        LRESULT(0)
      }
      WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
        if !IS_SYSTEM_SUSPENDED.load(Ordering::Relaxed) {
          if let Err(err) =
            handle_display_change_msg(message, wparam, event_tx)
          {
            warn!("Failed to handle display change message: {}", err);
          }
        }

        LRESULT(0)
      }
      WM_INPUT if ENABLE_MOUSE_EVENTS.load(Ordering::Relaxed) => {
        if let Err(err) = handle_input_msg(wparam, lparam, event_tx) {
          warn!("Failed to handle input message: {}", err);
        }

        LRESULT(0)
      }
      _ => unsafe { DefWindowProcW(handle, message, wparam, lparam) },
    };
  }

  LRESULT(0)
}

fn handle_display_change_msg(
  message: u32,
  wparam: WPARAM,
  event_tx: &mpsc::UnboundedSender<PlatformEvent>,
) -> anyhow::Result<()> {
  #[allow(clippy::cast_possible_truncation)]
  let should_emit_event = match message {
    WM_SETTINGCHANGE => {
      wparam.0 as u32 == SPI_SETWORKAREA.0
        || wparam.0 as u32 == SPI_ICONVERTICALSPACING.0
    }
    WM_DEVICECHANGE => wparam.0 as u32 == DBT_DEVNODES_CHANGED,
    _ => true,
  };

  if should_emit_event {
    event_tx.send(PlatformEvent::DisplaySettingsChanged)?;
  }

  Ok(())
}

fn handle_input_msg(
  _wparam: WPARAM,
  lparam: LPARAM,
  event_tx: &mpsc::UnboundedSender<PlatformEvent>,
) -> anyhow::Result<()> {
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
    flags if has_mouse_flag(flags, RI_MOUSE_LEFT_BUTTON_DOWN) => {
      IS_L_MOUSE_DOWN.store(true, Ordering::Relaxed);
      true
    }
    flags if has_mouse_flag(flags, RI_MOUSE_LEFT_BUTTON_UP) => {
      IS_L_MOUSE_DOWN.store(false, Ordering::Relaxed);
      true
    }
    flags if has_mouse_flag(flags, RI_MOUSE_RIGHT_BUTTON_DOWN) => {
      IS_R_MOUSE_DOWN.store(true, Ordering::Relaxed);
      true
    }
    flags if has_mouse_flag(flags, RI_MOUSE_RIGHT_BUTTON_UP) => {
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

  if !has_state_change && event_time - last_event_time < 50 {
    return Ok(());
  }

  let is_mouse_down = IS_L_MOUSE_DOWN.load(Ordering::Relaxed)
    || IS_R_MOUSE_DOWN.load(Ordering::Relaxed);

  let mut point = POINT { x: 0, y: 0 };
  unsafe { GetCursorPos(&raw mut point) }?;

  event_tx.send(PlatformEvent::MouseMove(MouseMoveEvent {
    point: Point {
      x: point.x,
      y: point.y,
    },
    is_mouse_down,
  }))?;

  LAST_MOUSE_EVENT_TIME.store(event_time, Ordering::Relaxed);

  Ok(())
}

#[inline]
fn has_mouse_flag(state: u16, mask: u32) -> bool {
  u32::from(state) & mask == mask
}