#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]

mod dispatcher;
mod display;
mod display_listener;
mod error;
mod event_loop;
mod keybinding_listener;
mod models;
mod mouse_listener;
mod native_window;
mod platform_event;
mod platform_impl;
mod single_instance;
mod thread_bound;
mod window_listener;

#[cfg(feature = "test_utils")]
pub mod test_utils;

pub use dispatcher::*;
pub use display::*;
pub use display_listener::*;
pub use error::*;
pub use event_loop::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
pub use native_window::*;
#[cfg(target_os = "windows")]
mod native_surrogate;
#[cfg(target_os = "windows")]
pub use native_surrogate::{NativeSurrogate, SurrogateBatch};
#[cfg(target_os = "windows")]
mod resize_session;
#[cfg(target_os = "windows")]
pub use resize_session::{ResizeSession, SessionOptions};
#[cfg(target_os = "windows")]
mod workspace_surrogate;
#[cfg(target_os = "windows")]
pub use workspace_surrogate::WorkspaceSurrogate;
#[cfg(target_os = "windows")]
mod native_iris_overlay;
#[cfg(target_os = "windows")]
pub use native_iris_overlay::NativeIrisOverlay;

pub use platform_event::*;
pub use single_instance::*;
pub use thread_bound::*;
pub use window_listener::*;
/// Waits for the next DWM composition frame to complete.
///
/// Used to synchronize animation ticks to vsync so surrogate updates reach
/// the compositor on every rendered frame without timer-resolution jitter.
/// On non-Windows platforms this is a no-op.
pub fn dwm_flush() {
  #[cfg(target_os = "windows")]
  unsafe {
    // SAFETY: No preconditions; `DwmFlush` is safe to call from any thread
    // and blocks until the next DWM composition frame is ready.
    let _ = windows::Win32::Graphics::Dwm::DwmFlush();
  }
}

/// Returns the current cursor position in virtual-screen pixels.
///
/// Returns `None` if the position cannot be queried. On non-Windows platforms
/// this always returns `None`.
pub fn cursor_position() -> Option<(i32, i32)> {
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::{
      Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos,
    };
    let mut point = POINT { x: 0, y: 0 };
    // SAFETY: `point` is a valid stack out-parameter.
    if unsafe { GetCursorPos(&raw mut point) }.is_ok() {
      Some((point.x, point.y))
    } else {
      None
    }
  }
  #[cfg(not(target_os = "windows"))]
  {
    None
  }
}

/// Per-monitor vsync waiter using `IDXGIOutput::WaitForVBlank`.
///
/// Unlike `DwmFlush`, which aligns to the primary monitor's global
/// composition cycle, this waits for the vertical-blank signal of a
/// specific monitor. Clones share the same atomics so all copies observe
/// period refinements and wake timestamps written by the timer thread.
#[cfg(target_os = "windows")]
#[derive(Clone)]
pub struct DxgiVsyncWaiter {
  output: windows::Win32::Graphics::Dxgi::IDXGIOutput,
  /// `HMONITOR` handle of the monitor this waiter was created for.
  monitor_handle: isize,
  /// Monitor refresh period in microseconds, shared across clones.
  ///
  /// Initialized from `query_frame_period_us`; refined downward in `wait` if
  /// the measured vblank delta is smaller (truer). Defaults to `16_667` (60 Hz).
  period_us: std::sync::Arc<std::sync::atomic::AtomicU64>,
  /// Timestamp of the most recent successful `WaitForVBlank` wake-up.
  ///
  /// Written by the timer thread immediately after vsync fires. Read by
  /// `predictive_vsync_now` to lead the animation clock by a fraction of a
  /// frame so the computed position aligns with the next DWM composition.
  pub last_wake: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

#[cfg(target_os = "windows")]
impl DxgiVsyncWaiter {
  /// Returns the `HMONITOR` handle this waiter was created for.
  #[must_use]
  pub fn monitor_handle(&self) -> isize {
    self.monitor_handle
  }

  /// Returns the current frame period estimate in microseconds.
  #[must_use]
  pub fn frame_period_us(&self) -> u64 {
    self.period_us.load(std::sync::atomic::Ordering::Relaxed)
  }

  /// Returns the `HMONITOR` handle of the monitor nearest to `hwnd`.
  #[must_use]
  pub fn window_monitor(hwnd: windows::Win32::Foundation::HWND) -> isize {
    use windows::Win32::Graphics::Gdi::{
      MonitorFromWindow, MONITOR_DEFAULTTONEAREST,
    };
    // SAFETY: `MonitorFromWindow` accepts any `HWND`; invalid handles yield
    // a null `HMONITOR`.
    unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) }.0
  }

  /// Locates the `IDXGIOutput` whose `HMONITOR` matches `monitor_handle`.
  ///
  /// Enumerates all DXGI adapters and their outputs. Returns `Err` when
  /// DXGI is unavailable or no output matches the given handle.
  pub fn for_monitor(monitor_handle: isize) -> crate::Result<Self> {
    use windows::Win32::{
      Graphics::{
        Dxgi::{CreateDXGIFactory, IDXGIFactory, DXGI_OUTPUT_DESC},
        Gdi::HMONITOR,
      },
    };

    // SAFETY: No preconditions for `CreateDXGIFactory`.
    let factory: IDXGIFactory = unsafe { CreateDXGIFactory()? };

    let mut ai = 0u32;
    loop {
      let Ok(adapter) = (unsafe { factory.EnumAdapters(ai) }) else {
        break; // DXGI_ERROR_NOT_FOUND â€” no more adapters.
      };
      let mut oi = 0u32;
      loop {
        let Ok(output) = (unsafe { adapter.EnumOutputs(oi) }) else {
          break; // No more outputs on this adapter.
        };
        let mut desc = DXGI_OUTPUT_DESC::default();
        // SAFETY: `output` is a valid `IDXGIOutput`; `desc` is stack-allocated
        // and passed as an out-parameter per the windows-rs 0.52 convention.
        if unsafe { output.GetDesc(&mut desc) }.is_ok()
          && desc.Monitor == HMONITOR(monitor_handle)
        {
          let period_us = Self::query_frame_period_us(&desc.DeviceName);
          return Ok(Self {
            output,
            monitor_handle,
            period_us: std::sync::Arc::new(
              std::sync::atomic::AtomicU64::new(period_us),
            ),
            last_wake: std::sync::Arc::new(std::sync::Mutex::new(None)),
          });
        }
        oi += 1;
      }
      ai += 1;
    }
    Err(crate::Error::DisplayNotFound)
  }

  /// Locates the `IDXGIOutput` for the monitor that `hwnd` currently occupies.
  ///
  /// Delegates to [`window_monitor`] then [`for_monitor`]. Returns `Err`
  /// when DXGI is unavailable or no output matches the window's monitor.
  ///
  /// [`window_monitor`]: DxgiVsyncWaiter::window_monitor
  /// [`for_monitor`]: DxgiVsyncWaiter::for_monitor
  pub fn for_window(
    hwnd: windows::Win32::Foundation::HWND,
  ) -> crate::Result<Self> {
    Self::for_monitor(Self::window_monitor(hwnd))
  }

  /// Queries the current refresh period (microseconds per vblank) for the GDI
  /// device named by `device_name`.
  ///
  /// Reads the active display mode via `EnumDisplaySettingsW`. Returns the
  /// 60 Hz period (`16_667`) when the rate is unavailable or reported as the
  /// hardware-default sentinel (`0` or `1`).
  fn query_frame_period_us(device_name: &[u16; 32]) -> u64 {
    use windows::{
      core::PCWSTR,
      Win32::Graphics::Gdi::{
        EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS,
      },
    };

    const DEFAULT_60HZ_US: u64 = 16_667;

    let mut devmode = DEVMODEW {
      dmSize: std::mem::size_of::<DEVMODEW>() as u16,
      ..Default::default()
    };

    // SAFETY: `device_name` is a null-terminated wide string from
    // `DXGI_OUTPUT_DESC`; `devmode` is stack-allocated with `dmSize` set per
    // the `EnumDisplaySettingsW` contract.
    let ok = unsafe {
      EnumDisplaySettingsW(
        PCWSTR(device_name.as_ptr()),
        ENUM_CURRENT_SETTINGS,
        &mut devmode,
      )
    }
    .as_bool();

    if !ok || devmode.dmDisplayFrequency <= 1 {
      return DEFAULT_60HZ_US;
    }

    1_000_000 / u64::from(devmode.dmDisplayFrequency)
  }

  /// Blocks until the next vertical-blank signal from this output.
  ///
  /// On success, records the wake timestamp in `last_wake` and ratchets
  /// `period_us` downward if the measured delta is plausible (25-370 Hz).
  /// Returns `true` on success, `false` on error.
  pub fn wait(&self) -> bool {
    // SAFETY: `self.output` is a valid `IDXGIOutput` kept alive by the
    // `Clone`-counted reference.
    if unsafe { self.output.WaitForVBlank() }.is_err() {
      return false;
    }
    let now = std::time::Instant::now();
    if let Ok(mut guard) = self.last_wake.lock() {
      if let Some(prev) = *guard {
        let delta_us = u64::try_from(now.duration_since(prev).as_micros())
          .unwrap_or(u64::MAX);
        // Plausible vblank window: 25-370 Hz -> 2_700-40_000 us.
        if (2_700..=40_000).contains(&delta_us) {
          let current =
            self.period_us.load(std::sync::atomic::Ordering::Relaxed);
          if delta_us < current {
            self
              .period_us
              .store(delta_us, std::sync::atomic::Ordering::Relaxed);
          }
        }
      }
      *guard = Some(now);
    }
    true
  }
}

/// RAII guard that reverts an MMCSS thread registration on drop.
///
/// Obtain via [`try_set_thread_mmcss`]. Dropping this guard calls
/// `AvRevertMmThread`, restoring the thread to normal scheduling. This
/// ensures cleanup even if the animation thread exits through an early-return
/// path.
#[cfg(target_os = "windows")]
pub struct MmcssGuard(isize);

#[cfg(target_os = "windows")]
impl Drop for MmcssGuard {
  fn drop(&mut self) {
    use windows::Win32::System::LibraryLoader::{
      GetModuleHandleW, GetProcAddress,
    };

    type AvRevertFn = unsafe extern "system" fn(isize) -> i32;

    static FN: std::sync::OnceLock<Option<AvRevertFn>> =
      std::sync::OnceLock::new();

    let Some(f) = *FN.get_or_init(|| {
      // SAFETY: avrt.dll was already loaded by `try_set_thread_mmcss`.
      unsafe {
        let module =
          GetModuleHandleW(windows::core::w!("avrt.dll")).ok()?;
        let proc =
          GetProcAddress(module, windows::core::s!("AvRevertMmThread"))?;
        Some(std::mem::transmute::<
          unsafe extern "system" fn() -> isize,
          AvRevertFn,
        >(proc))
      }
    }) else {
      return;
    };

    // SAFETY: `self.0` is a valid AVRT handle from `AvSetMmThreadCharacteristicsW`.
    unsafe { f(self.0) };
  }
}

/// Registers the calling thread with the Multimedia Class Scheduler Service
/// (MMCSS) for display post-processing.
///
/// MMCSS gives the thread near-real-time scheduling guarantees beyond
/// `THREAD_PRIORITY_HIGHEST`, reducing OS scheduling jitter after a vsync
/// wake-up. This is the mechanism Windows uses internally for DWM, video
/// players, and game render threads.
///
/// Returns a [`MmcssGuard`] that automatically reverts the registration on
/// drop, or `None` if `avrt.dll` is unavailable or registration fails.
#[cfg(target_os = "windows")]
pub fn try_set_thread_mmcss() -> Option<MmcssGuard> {
  use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryW,
  };

  type AvSetMmFn =
    unsafe extern "system" fn(*const u16, *mut u32) -> isize;

  static FN: std::sync::OnceLock<Option<AvSetMmFn>> =
    std::sync::OnceLock::new();

  let f = (*FN.get_or_init(|| {
    // SAFETY: `avrt.dll` is a standard system library present on Vista+.
    unsafe {
      let module =
        LoadLibraryW(windows::core::w!("avrt.dll")).ok()?;
      let proc = GetProcAddress(
        module,
        windows::core::s!("AvSetMmThreadCharacteristicsW"),
      )?;
      Some(std::mem::transmute::<
        unsafe extern "system" fn() -> isize,
        AvSetMmFn,
      >(proc))
    }
  }))?;

  // "DisplayPostProcessing" is the MMCSS task class used by DWM and video
  // renderers. It grants near-real-time scheduling priority.
  let task: Vec<u16> = "DisplayPostProcessing\0".encode_utf16().collect();
  let mut idx: u32 = 0;

  // SAFETY: `task` is a null-terminated wide string; `idx` is a valid
  // stack-allocated output parameter.
  let handle = unsafe { f(task.as_ptr(), &mut idx) };

  if handle != 0 {
    Some(MmcssGuard(handle))
  } else {
    None
  }
}

/// Sets the calling thread's scheduling priority to highest.
///
/// Called at the start of the animation timer thread to reduce scheduling
/// jitter between the DWM VSync wake-up and tick delivery to the Tokio
/// runtime. On non-Windows platforms this is a no-op.
pub fn set_thread_priority_highest() {
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::System::Threading::{
      GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
    };
    // SAFETY: `GetCurrentThread` returns a pseudo-handle valid for the
    // lifetime of the calling thread. `SetThreadPriority` has no
    // preconditions beyond a valid handle and a recognised priority value.
    unsafe {
      let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
    }
  }
}

// TODO: Avoid exposing `windows` crate types in the public API.
#[cfg(target_os = "windows")]
pub use windows::Win32::UI::WindowsAndMessaging::{
  SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
  SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, SWP_NOZORDER,
  WINDOW_EX_STYLE, WINDOW_STYLE, WS_CAPTION, WS_CHILD, WS_EX_NOACTIVATE,
  WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX,
};
