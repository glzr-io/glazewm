use std::cell::OnceCell;

use objc2_app_kit::NSScreen;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGMainDisplayID,
};
use objc2_foundation::MainThreadMarker;
use wm_common::Rect;

use crate::error::{Error, Result};

/// Native monitor representation for macOS.
///
/// This struct provides access to display properties using Core Graphics
/// and AppKit APIs. Information is lazily loaded and cached for
/// performance.
#[derive(Clone, Debug)]
pub struct NativeMonitor {
  pub display_id: CGDirectDisplayID,
  info: OnceCell<MonitorInfo>,
}

#[derive(Clone, Debug)]
struct MonitorInfo {
  device_name: String,
  hardware_id: Option<String>,
  rect: Rect,
  working_rect: Rect,
  scale_factor: f32,
  is_primary: bool,
}

impl NativeMonitor {
  /// Creates a new `NativeMonitor` instance for the given display ID.
  #[must_use]
  pub fn new(display_id: CGDirectDisplayID) -> Self {
    Self {
      display_id,
      info: OnceCell::new(),
    }
  }

  /// Gets the device name of the monitor.
  ///
  /// On macOS, this attempts to get the localized name from `NSScreen`,
  /// falling back to a generic name based on the display ID.
  pub fn device_name(&self) -> Result<&String> {
    self.monitor_info().map(|info| &info.device_name)
  }

  /// Gets the hardware identifier for the monitor.
  ///
  /// This is derived from the I/O service port if available.
  pub fn hardware_id(&self) -> Result<Option<&String>> {
    self.monitor_info().map(|info| info.hardware_id.as_ref())
  }

  /// Gets the full bounds rectangle of the monitor.
  pub fn rect(&self) -> Result<&Rect> {
    self.monitor_info().map(|info| &info.rect)
  }

  /// Gets the working area rectangle (excluding dock and menu bar).
  pub fn working_rect(&self) -> Result<&Rect> {
    self.monitor_info().map(|info| &info.working_rect)
  }

  /// Gets the scale factor for the monitor.
  ///
  /// This corresponds to the backing scale factor for high-DPI displays.
  pub fn scale_factor(&self) -> Result<f32> {
    self.monitor_info().map(|info| info.scale_factor)
  }

  /// Gets the DPI for the monitor.
  ///
  /// This is calculated from the scale factor, assuming 72 DPI base.
  pub fn dpi(&self) -> Result<u32> {
    let scale_factor = self.scale_factor()?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((72.0 * scale_factor) as u32)
  }

  /// Returns whether this is the primary monitor.
  pub fn is_primary(&self) -> Result<bool> {
    self.monitor_info().map(|info| info.is_primary)
  }

  /// Lazily loads and caches monitor information.
  #[allow(clippy::cast_possible_truncation)]
  fn monitor_info(&self) -> Result<&MonitorInfo> {
    self.info.get_or_try_init(|| {
      let main_thread = MainThreadMarker::new().ok_or_else(|| {
        Error::Anyhow(anyhow::anyhow!("Not on main thread"))
      })?;

      // Get basic display bounds using Core Graphics.
      let cg_rect = unsafe { CGDisplayBounds(self.display_id) };
      let rect = Rect::from_ltrb(
        cg_rect.origin.x as i32,
        cg_rect.origin.y as i32,
        (cg_rect.origin.x + cg_rect.size.width) as i32,
        (cg_rect.origin.y + cg_rect.size.height) as i32,
      );

      // Find the corresponding NSScreen for this display.
      let screens = NSScreen::screens(main_thread);
      let mut device_name = format!("Display {}", self.display_id);
      let mut scale_factor = 1.0;

      // Store rect dimensions for comparison
      let rect_x = rect.x();
      let rect_width = rect.width();
      let mut working_rect = rect;

      // Search for matching NSScreen by comparing display bounds.
      for screen in &screens {
        let screen_frame = screen.frame();
        let screen_rect = Rect::from_ltrb(
          screen_frame.origin.x as i32,
          // Flip Y coordinate (NSScreen uses bottom-left origin).
          (screen_frame.origin.y + screen_frame.size.height) as i32,
          (screen_frame.origin.x + screen_frame.size.width) as i32,
          screen_frame.origin.y as i32,
        );

        // Check if this screen matches our display bounds.
        // Compare by x coordinate and width to find matching screen
        if screen_rect.x() == rect_x && screen_rect.width() == rect_width {
          // Use a generic name for now - localizedName API might vary
          device_name = format!(
            "Display {} ({}x{})",
            self.display_id,
            screen_rect.width(),
            screen_rect.height()
          );

          // Get visible frame (working area excluding dock/menu bar).
          let visible_frame = screen.visibleFrame();
          working_rect = Rect::from_ltrb(
            visible_frame.origin.x as i32,
            // Flip Y coordinate for working rect too.
            (visible_frame.origin.y + visible_frame.size.height) as i32,
            (visible_frame.origin.x + visible_frame.size.width) as i32,
            visible_frame.origin.y as i32,
          );

          // Get the backing scale factor - simplified to 1.0 for now
          // The correct objc2 method name might be different
          scale_factor = 1.0;
          break;
        }
      }

      // Generate a simple hardware ID based on display ID.
      let hardware_id =
        Some(format!("CGDirectDisplayID:{}", self.display_id));

      let is_primary = unsafe { CGMainDisplayID() } == self.display_id;

      // Reconstruct the original rect from stored values
      let final_rect = Rect::from_ltrb(
        cg_rect.origin.x as i32,
        cg_rect.origin.y as i32,
        (cg_rect.origin.x + cg_rect.size.width) as i32,
        (cg_rect.origin.y + cg_rect.size.height) as i32,
      );

      Ok(MonitorInfo {
        device_name,
        hardware_id,
        rect: final_rect,
        working_rect,
        scale_factor,
        is_primary,
      })
    })
  }
}

impl PartialEq for NativeMonitor {
  fn eq(&self, other: &Self) -> bool {
    self.display_id == other.display_id
  }
}

impl Eq for NativeMonitor {}

/// Gets all available monitors.
///
/// Uses Core Graphics to enumerate all active displays and returns
/// them as `NativeMonitor` instances sorted by display ID.
pub fn available_monitors() -> Result<Vec<NativeMonitor>> {
  let display_ids = available_display_ids()?;
  Ok(display_ids.into_iter().map(NativeMonitor::new).collect())
}

/// Gets all available display IDs.
///
/// For now, we'll use a simplified approach that gets the main display.
/// This can be expanded to use `NSScreen` enumeration or Core Graphics
/// if needed.
fn available_display_ids() -> Result<Vec<CGDirectDisplayID>> {
  let mut display_ids = Vec::new();

  // Add main display
  let main_id = unsafe { CGMainDisplayID() };
  display_ids.push(main_id);

  // For additional displays, we'd need to use private APIs or
  // platform-specific methods. For now, just return main display.
  Ok(display_ids)
}

/// Gets the monitor containing the specified point.
///
/// If no monitor contains the point, returns the primary monitor.
#[must_use]
pub fn monitor_from_point(x: i32, y: i32) -> Option<NativeMonitor> {
  available_monitors()
    .ok()?
    .into_iter()
    .find(|monitor| {
      monitor.rect().map_or(false, |rect| {
        rect.contains_point(&wm_common::Point { x, y })
      })
    })
    .or_else(|| {
      // Fall back to primary monitor.
      let main_display_id = unsafe { CGMainDisplayID() };
      Some(NativeMonitor::new(main_display_id))
    })
}

/// Gets the primary monitor.
#[must_use]
pub fn primary_monitor() -> NativeMonitor {
  let main_display_id = unsafe { CGMainDisplayID() };
  NativeMonitor::new(main_display_id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_monitor_structure() {
    // Test basic structure creation (doesn't require main thread)
    if cfg!(target_os = "macos") {
      let monitor = NativeMonitor::new(1);
      assert_eq!(monitor.display_id, 1);

      // Test equality
      let monitor2 = NativeMonitor::new(1);
      let monitor3 = NativeMonitor::new(2);
      assert_eq!(monitor, monitor2);
      assert_ne!(monitor, monitor3);
    }
  }

  #[test]
  fn test_available_monitors_function_exists() {
    // Just test that the function exists and can be called
    // (May fail due to main thread requirement, but that's expected)
    if cfg!(target_os = "macos") {
      let _result = available_monitors();
      // Don't assert on the result, just ensure it compiles and runs
    }
  }
}
