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
pub use native_surrogate::NativeSurrogate;
#[cfg(target_os = "windows")]
mod resize_session;
#[cfg(target_os = "windows")]
pub use resize_session::ResizeSession;
#[cfg(target_os = "windows")]
mod workspace_surrogate;
#[cfg(target_os = "windows")]
pub use workspace_surrogate::WorkspaceSurrogate;

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

/// Per-monitor vsync waiter using `IDXGIOutput::WaitForVBlank`.
///
/// Unlike `DwmFlush`, which aligns to the primary monitor's global
/// composition cycle, this waits for the vertical-blank signal of a
/// specific monitor. This gives the animation tick thread a full frame
/// period to update surrogates before the next DWM composition, regardless
/// of which monitor is the Windows primary and regardless of whether
/// multiple monitors with different refresh rates are connected.
#[cfg(target_os = "windows")]
#[derive(Clone)]
pub struct DxgiVsyncWaiter {
  output: windows::Win32::Graphics::Dxgi::IDXGIOutput,
}

#[cfg(target_os = "windows")]
impl DxgiVsyncWaiter {
  /// Locates the `IDXGIOutput` whose `HMONITOR` matches `monitor_handle`.
  ///
  /// Enumerates all DXGI adapters and their outputs. Returns `None` when
  /// DXGI is unavailable or no output matches the given handle.
  pub fn for_monitor(monitor_handle: isize) -> Option<Self> {
    use windows::Win32::{
      Graphics::{
        Dxgi::{CreateDXGIFactory, IDXGIFactory, DXGI_OUTPUT_DESC},
        Gdi::HMONITOR,
      },
    };

    // SAFETY: No preconditions for `CreateDXGIFactory`.
    let factory: IDXGIFactory = unsafe { CreateDXGIFactory().ok()? };

    let mut ai = 0u32;
    loop {
      let Ok(adapter) = (unsafe { factory.EnumAdapters(ai) }) else {
        break; // DXGI_ERROR_NOT_FOUND — no more adapters.
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
          return Some(Self { output });
        }
        oi += 1;
      }
      ai += 1;
    }
    None
  }

  /// Blocks until the next vertical-blank signal from this output.
  ///
  /// Returns `true` on success, `false` on error (caller should fall back
  /// to an alternative wait strategy).
  pub fn wait(&self) -> bool {
    // SAFETY: `self.output` is a valid `IDXGIOutput` kept alive by the
    // `Clone`-counted reference.
    unsafe { self.output.WaitForVBlank().is_ok() }
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
