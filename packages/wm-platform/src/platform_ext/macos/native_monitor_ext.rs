#[cfg(target_os = "macos")]
use objc2_app_kit::NSScreen;
#[cfg(target_os = "macos")]
use objc2_core_graphics::CGDirectDisplayID;
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;

#[cfg(target_os = "macos")]
use crate::{
  platform_impl::EventLoopDispatcher, NativeMonitor, Result,
};

/// macOS-specific extensions for `NativeMonitor`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on macOS.
#[cfg(target_os = "macos")]
pub trait NativeMonitorExtMacos {
  /// Gets the Core Graphics display ID.
  fn display_id(&self) -> CGDirectDisplayID;

  /// Gets the `NSScreen` instance for this monitor.
  ///
  /// This method requires main thread access and will use the event
  /// loop dispatcher to ensure thread safety.
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn ns_screen(&self) -> Result<Option<objc2::rc::Retained<NSScreen>>>;
}

#[cfg(target_os = "macos")]
impl NativeMonitorExtMacos for NativeMonitor {
  fn display_id(&self) -> CGDirectDisplayID {
    self.inner.display_id
  }

  fn ns_screen(&self) -> Result<Option<objc2::rc::Retained<NSScreen>>> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.ns_screen_on_main_thread(mtm)
    })
  }
}

#[cfg(target_os = "macos")]
impl NativeMonitor {
  /// Gets the NSScreen on the main thread.
  fn ns_screen_on_main_thread(
    &self,
    mtm: MainThreadMarker,
  ) -> Result<Option<objc2::rc::Retained<NSScreen>>> {
    let screens = NSScreen::screens(mtm);
    let rect = self.rect()?;

    // Find the corresponding NSScreen by comparing bounds
    for screen in screens {
      let screen_frame = screen.frame();
      let screen_rect = wm_common::Rect::from_ltrb(
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.x as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.y + screen_frame.size.height) as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.x + screen_frame.size.width) as i32,
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.y as i32,
      );

      if screen_rect.x() == rect.x() && screen_rect.width() == rect.width() {
        return Ok(Some(screen));
      }
    }

    Ok(None)
  }
}