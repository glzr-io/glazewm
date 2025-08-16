use objc2_app_kit::NSScreen;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayMirrorsDisplay,
  CGGetActiveDisplayList, CGGetOnlineDisplayList, CGMainDisplayID,
};
use objc2_foundation::MainThreadMarker;
use wm_common::{Point, Rect};

use crate::{
  error::{Error, Result},
  platform_impl::EventLoopDispatcher,
  MonitorState,
};

/// macOS-specific monitor implementation.
#[derive(Clone, Debug)]
pub struct NativeMonitor {
  pub display_id: CGDirectDisplayID,
  dispatcher: EventLoopDispatcher,
}

impl NativeMonitor {
  /// Creates a new `NativeMonitor` for the given display ID.
  #[must_use]
  pub fn new(
    display_id: CGDirectDisplayID,
    dispatcher: &EventLoopDispatcher,
  ) -> Self {
    Self {
      display_id,
      dispatcher: dispatcher.clone(),
    }
  }

  /// Gets the device name of the monitor.
  pub fn device_name(&self) -> Result<String> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.device_name_on_main_thread(mtm)
    })
  }

  /// Gets the hardware identifier for the monitor.
  pub fn hardware_id(&self) -> Result<Option<String>> {
    // Generate a hardware ID based on display ID
    Ok(Some(format!("CGDirectDisplayID:{}", self.display_id)))
  }

  /// Gets the full bounds rectangle of the monitor.
  pub fn rect(&self) -> Result<Rect> {
    let cg_rect = unsafe { CGDisplayBounds(self.display_id) };

    #[allow(clippy::cast_possible_truncation)]
    Ok(Rect::from_ltrb(
      cg_rect.origin.x as i32,
      cg_rect.origin.y as i32,
      (cg_rect.origin.x + cg_rect.size.width) as i32,
      (cg_rect.origin.y + cg_rect.size.height) as i32,
    ))
  }

  /// Gets the working area rectangle (excluding dock and menu bar).
  pub fn working_rect(&self) -> Result<Rect> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.working_rect_on_main_thread(mtm)
    })
  }

  /// Gets the DPI for the monitor.
  pub fn dpi(&self) -> Result<u32> {
    let scale_factor = self.scale_factor()?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((72.0 * scale_factor) as u32)
  }

  /// Gets the scale factor for the monitor.
  pub fn scale_factor(&self) -> Result<f32> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.scale_factor_on_main_thread(mtm)
    })
  }

  /// Gets the current state of the monitor.
  pub fn state(&self) -> Result<MonitorState> {
    // Check if this display is mirroring another
    let mirrors_display =
      unsafe { CGDisplayMirrorsDisplay(self.display_id) };
    if mirrors_display != 0 {
      return Ok(MonitorState::Mirroring);
    }

    // Check if display is in the active list
    let active_displays = get_active_display_ids()?;
    if active_displays.contains(&self.display_id) {
      Ok(MonitorState::Active)
    } else {
      Ok(MonitorState::Inactive)
    }
  }

  /// Returns whether this is the primary monitor.
  pub fn is_primary(&self) -> Result<bool> {
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.display_id == main_display_id)
  }

  /// Gets the device name on the main thread.
  fn device_name_on_main_thread(
    &self,
    mtm: MainThreadMarker,
  ) -> Result<String> {
    let screens = NSScreen::screens(mtm);
    let rect = self.rect()?;

    // Find the corresponding NSScreen by comparing bounds.
    for screen in &screens {
      let screen_frame = screen.frame();

      #[allow(clippy::cast_possible_truncation)]
      let screen_rect = Rect::from_ltrb(
        screen_frame.origin.x as i32,
        // Flip Y coordinate (NSScreen uses bottom-left origin).
        (screen_frame.origin.y + screen_frame.size.height) as i32,
        (screen_frame.origin.x + screen_frame.size.width) as i32,
        screen_frame.origin.y as i32,
      );

      if screen_rect.x() == rect.x() && screen_rect.width() == rect.width()
      {
        return Ok(format!(
          "Display {} ({}×{})",
          self.display_id,
          rect.width(),
          rect.height()
        ));
      }
    }

    Ok(format!("Display {}", self.display_id))
  }

  /// Gets the working rect on the main thread.
  fn working_rect_on_main_thread(
    &self,
    mtm: MainThreadMarker,
  ) -> Result<Rect> {
    let screens = NSScreen::screens(mtm);
    let rect = self.rect()?;

    // Find the corresponding NSScreen by comparing bounds
    for screen in &screens {
      let screen_frame = screen.frame();
      let screen_rect = Rect::from_ltrb(
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.x as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.y + screen_frame.size.height) as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.x + screen_frame.size.width) as i32,
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.y as i32,
      );

      if screen_rect.x() == rect.x() && screen_rect.width() == rect.width()
      {
        let visible_frame = screen.visibleFrame();
        return Ok(Rect::from_ltrb(
          #[allow(clippy::cast_possible_truncation)]
          visible_frame.origin.x as i32,
          #[allow(clippy::cast_possible_truncation)]
          (visible_frame.origin.y + visible_frame.size.height)
            as i32,
          #[allow(clippy::cast_possible_truncation)]
          (visible_frame.origin.x + visible_frame.size.width)
            as i32,
          #[allow(clippy::cast_possible_truncation)]
          visible_frame.origin.y as i32,
        ));
      }
    }

    // If no NSScreen found, return the full rect
    Ok(rect)
  }

  /// Gets the scale factor on the main thread.
  fn scale_factor_on_main_thread(
    &self,
    mtm: MainThreadMarker,
  ) -> Result<f32> {
    let screens = NSScreen::screens(mtm);
    let rect = self.rect()?;

    // Find the corresponding NSScreen by comparing bounds
    for screen in &screens {
      let screen_frame = screen.frame();
      let screen_rect = Rect::from_ltrb(
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.x as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.y + screen_frame.size.height) as i32,
        #[allow(clippy::cast_possible_truncation)]
        (screen_frame.origin.x + screen_frame.size.width) as i32,
        #[allow(clippy::cast_possible_truncation)]
        screen_frame.origin.y as i32,
      );

      if screen_rect.x() == rect.x() && screen_rect.width() == rect.width()
      {
        #[allow(clippy::cast_possible_truncation)]
        return Ok(screen.backingScaleFactor() as f32);
      }
    }

    // Default to 1.0 if no matching NSScreen found
    Ok(1.0)
  }
}

/// Gets all monitors, including active, mirroring, and inactive ones.
pub fn all_monitors() -> Result<Vec<NativeMonitor>> {
  let display_ids = get_all_display_ids()?;
  Ok(display_ids.into_iter().map(NativeMonitor::new).collect())
}

/// Gets the monitor containing the specified point.
pub fn monitor_from_point(point: Point) -> Result<NativeMonitor> {
  let monitors = all_monitors()?;

  for monitor in &monitors {
    let rect = monitor.rect()?;
    if rect.contains_point(&point) {
      return Ok(monitor.clone());
    }
  }

  // Fall back to primary monitor
  primary_monitor()
}

/// Gets the primary monitor.
pub fn primary_monitor() -> Result<NativeMonitor> {
  let main_display_id = unsafe { CGMainDisplayID() };
  Ok(NativeMonitor::new(main_display_id))
}

/// Gets all display IDs (active and inactive).
fn get_all_display_ids() -> Result<Vec<CGDirectDisplayID>> {
  let mut display_count = 0u32;

  // First get the count
  let result = unsafe {
    CGGetOnlineDisplayList(0, std::ptr::null_mut(), &mut display_count)
  };

  if result != 0 {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get display count: {}",
      result
    )));
  }

  if display_count == 0 {
    return Ok(Vec::new());
  }

  // Then get the actual display IDs
  let mut display_ids = vec![0; display_count as usize];
  let result = unsafe {
    CGGetOnlineDisplayList(
      display_count,
      display_ids.as_mut_ptr(),
      &mut display_count,
    )
  };

  if result != 0 {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get display list: {}",
      result
    )));
  }

  Ok(display_ids)
}

/// Gets active display IDs.
fn get_active_display_ids() -> Result<Vec<CGDirectDisplayID>> {
  let mut display_count = 0u32;

  // First get the count
  let result = unsafe {
    CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut display_count)
  };

  if result != 0 {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get active display count: {}",
      result
    )));
  }

  if display_count == 0 {
    return Ok(Vec::new());
  }

  // Then get the actual display IDs
  let mut display_ids = vec![0; display_count as usize];
  let result = unsafe {
    CGGetActiveDisplayList(
      display_count,
      display_ids.as_mut_ptr(),
      &mut display_count,
    )
  };

  if result != 0 {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get active display list: {}",
      result
    )));
  }

  Ok(display_ids)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_monitor_creation() {
    let monitor = NativeMonitor::new(1);
    assert_eq!(monitor.display_id, 1);

    let monitor2 = NativeMonitor::new(1);
    let monitor3 = NativeMonitor::new(2);
    assert_eq!(monitor, monitor2);
    assert_ne!(monitor, monitor3);
  }

  #[test]
  fn test_hardware_id() {
    let monitor = NativeMonitor::new(42);
    let hardware_id = monitor.hardware_id().unwrap();
    assert_eq!(hardware_id, Some("CGDirectDisplayID:42".to_string()));
  }
}
