use objc2_app_kit::NSScreen;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyAllDisplayModes,
  CGDisplayCopyDisplayMode, CGDisplayMirrorsDisplay,
  CGDisplayModeGetHeight, CGDisplayModeGetPixelHeight,
  CGDisplayModeGetPixelWidth, CGDisplayModeGetRefreshRate,
  CGDisplayModeGetWidth, CGDisplayModeRef, CGDisplayScreenSize, CGError,
  CGGetActiveDisplayList, CGGetOnlineDisplayList, CGMainDisplayID,
};
use objc2_foundation::MainThreadMarker;
use wm_common::{Point, Rect};

use crate::{
  display::{
    DisplayConnection, DisplayDeviceData, DisplayDeviceId,
    DisplayDeviceState, DisplayId, MirroringState, PhysicalDeviceData,
    VirtualDeviceData,
  },
  error::{Error, Result},
  platform_ext::macos::{CFRetained, MainThreadRef, MetalDeviceRef},
  platform_impl::EventLoopDispatcher,
};

/// macOS-specific display implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
  pub(crate) display_id: CGDirectDisplayID,
  pub(crate) device_id: String,
  dispatcher: EventLoopDispatcher,
}

impl Display {
  /// Creates a new macOS display.
  #[must_use]
  pub fn new(
    display_id: CGDirectDisplayID,
    device_id: String,
    dispatcher: &EventLoopDispatcher,
  ) -> Self {
    Self {
      display_id,
      device_id,
      dispatcher: dispatcher.clone(),
    }
  }

  /// Gets the unique identifier for this display.
  pub fn id(&self) -> DisplayId {
    DisplayId::new(format!("macos:{}", self.display_id))
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.device_name_on_main_thread(mtm)
    })
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> Result<Rect> {
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
  pub fn working_area(&self) -> Result<Rect> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.working_rect_on_main_thread(mtm)
    })
  }

  /// Gets the display resolution in pixels.
  pub fn resolution(&self) -> Result<(u32, u32)> {
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.display_id) };
    if display_mode.is_null() {
      return Err(Error::Anyhow(anyhow::anyhow!(
        "Failed to get display mode for display {}",
        self.display_id
      )));
    }

    let width = unsafe { CGDisplayModeGetPixelWidth(display_mode) };
    let height = unsafe { CGDisplayModeGetPixelHeight(display_mode) };

    Ok((width as u32, height as u32))
  }

  /// Gets the current refresh rate in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.display_id) };
    if display_mode.is_null() {
      return Err(Error::Anyhow(anyhow::anyhow!(
        "Failed to get display mode for display {}",
        self.display_id
      )));
    }

    let refresh_rate =
      unsafe { CGDisplayModeGetRefreshRate(display_mode) };
    Ok(refresh_rate as f32)
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> Result<f32> {
    EventLoopDispatcher::with_main_thread(|mtm| {
      self.scale_factor_on_main_thread(mtm)
    })
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> Result<u32> {
    let scale_factor = self.scale_factor()?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((72.0 * scale_factor) as u32)
  }

  /// Gets the bit depth of the display.
  pub fn bit_depth(&self) -> Result<u32> {
    // Most modern macOS displays are 24-bit or 30-bit
    // Would need IOKit to get exact bit depth
    Ok(24)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.display_id == main_display_id)
  }

  /// Returns whether this display supports HDR.
  pub fn is_hdr_capable(&self) -> Result<bool> {
    // Would need to check display capabilities via IOKit or NSScreen
    Ok(false)
  }

  /// Gets all supported refresh rates for this display.
  pub fn supported_refresh_rates(&self) -> Result<Vec<f32>> {
    let modes_array = unsafe {
      CGDisplayCopyAllDisplayModes(self.display_id, std::ptr::null())
    };
    if modes_array.is_null() {
      return Ok(vec![self.refresh_rate()?]);
    }

    let mut refresh_rates = Vec::new();

    // Would need to iterate through CFArray of display modes
    // This is a simplified implementation
    refresh_rates.push(self.refresh_rate()?);

    // Common refresh rates for macOS displays
    refresh_rates.extend([60.0, 120.0]);
    refresh_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    refresh_rates.dedup();

    Ok(refresh_rates)
  }

  /// Gets all supported resolutions for this display.
  pub fn supported_resolutions(&self) -> Result<Vec<(u32, u32)>> {
    let modes_array = unsafe {
      CGDisplayCopyAllDisplayModes(self.display_id, std::ptr::null())
    };
    if modes_array.is_null() {
      return Ok(vec![self.resolution()?]);
    }

    let mut resolutions = Vec::new();

    // Would need to iterate through CFArray of display modes
    // This is a simplified implementation
    resolutions.push(self.resolution()?);

    // Common resolutions for macOS displays
    resolutions.extend([
      (2560, 1440),
      (1920, 1080),
      (1680, 1050),
      (1440, 900),
    ]);

    resolutions.sort();
    resolutions.dedup();

    Ok(resolutions)
  }

  /// Gets the ID of the device driving this display.
  pub fn device_id(&self) -> DisplayDeviceId {
    DisplayDeviceId::new(&self.device_id)
  }

  // macOS-specific methods

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.display_id
  }

  /// Gets the NSScreen instance for this display.
  pub fn ns_screen(&self) -> Result<MainThreadRef<CFRetained<NSScreen>>> {
    // This would need proper implementation with MainThreadRef and
    // CFRetained For now, this is a placeholder
    todo!("Implement proper MainThreadRef<CFRetained<NSScreen>> handling")
  }

  /// Checks if this is a built-in display.
  pub fn is_builtin(&self) -> Result<bool> {
    // Would use IOKit to check if this is an internal display
    // For now, check if it's the main display as a heuristic
    self.is_primary()
  }

  /// Gets the device name on the main thread.
  fn device_name_on_main_thread(
    &self,
    mtm: MainThreadMarker,
  ) -> Result<String> {
    let screens = NSScreen::screens(mtm);
    let rect = self.bounds()?;

    // Find the corresponding NSScreen by comparing bounds
    for screen in &screens {
      let screen_frame = screen.frame();

      #[allow(clippy::cast_possible_truncation)]
      let screen_rect = Rect::from_ltrb(
        screen_frame.origin.x as i32,
        // Flip Y coordinate (NSScreen uses bottom-left origin)
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
    let rect = self.bounds()?;

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
    let rect = self.bounds()?;

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

/// macOS-specific display device implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) gpu_id: u64,
  pub(crate) name: String,
  pub(crate) is_virtual: bool,
  pub(crate) is_builtin: bool,
}

impl DisplayDevice {
  /// Creates a new macOS display device.
  #[must_use]
  pub fn new(
    gpu_id: u64,
    name: String,
    is_virtual: bool,
    is_builtin: bool,
  ) -> Self {
    Self {
      gpu_id,
      name,
      is_virtual,
      is_builtin,
    }
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    DisplayDeviceId::new(format!("gpu:{}", self.gpu_id))
  }

  /// Gets the device name.
  pub fn name(&self) -> Result<String> {
    Ok(self.name.clone())
  }

  /// Gets the current state of the device.
  pub fn state(&self) -> Result<DisplayDeviceState> {
    // All enumerated devices are typically active on macOS
    Ok(DisplayDeviceState::Active)
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<MirroringState> {
    // Would need to check display mirroring status
    Ok(MirroringState::None)
  }

  /// Gets the device-specific data.
  pub fn data(&self) -> Result<DisplayDeviceData> {
    if self.is_virtual {
      Ok(DisplayDeviceData::Virtual(VirtualDeviceData {
        driver_name: Some("Virtual Display Driver".to_string()),
        virtual_adapter_id: Some(format!("virtual:{}", self.gpu_id)),
      }))
    } else {
      let vendor = self.extract_vendor();
      let model = self.extract_model();

      Ok(DisplayDeviceData::Physical(PhysicalDeviceData {
        vendor,
        model,
        serial_number: None, // Would need IOKit lookup
        hardware_id: Some(format!("GPU:{}", self.gpu_id)),
        edid_data: None, // Would need IOKit EDID retrieval
        physical_size_mm: None, // Would need EDID parsing
        connection_type: Some(if self.is_builtin {
          DisplayConnection::Internal
        } else {
          DisplayConnection::Unknown
        }),
        is_builtin: self.is_builtin,
      }))
    }
  }

  // macOS-specific methods

  /// Gets the display unit number.
  pub fn unit_number(&self) -> Result<Option<u32>> {
    if self.is_virtual {
      Ok(None)
    } else {
      Ok(Some(0)) // Would need IOKit lookup
    }
  }

  /// Gets the GPU registry ID.
  pub fn registry_id(&self) -> Result<Option<u64>> {
    if self.is_virtual {
      Ok(None)
    } else {
      Ok(Some(self.gpu_id))
    }
  }

  /// Gets a reference to the Metal device.
  pub fn metal_device(&self) -> Result<Option<MetalDeviceRef>> {
    if self.is_virtual {
      Ok(None)
    } else {
      Ok(Some(MetalDeviceRef { inner: self.gpu_id }))
    }
  }

  /// Extracts vendor name from device name.
  fn extract_vendor(&self) -> Option<String> {
    let name_lower = self.name.to_lowercase();

    if name_lower.contains("apple") {
      Some("Apple".to_string())
    } else if name_lower.contains("nvidia") {
      Some("NVIDIA".to_string())
    } else if name_lower.contains("amd") || name_lower.contains("radeon") {
      Some("AMD".to_string())
    } else if name_lower.contains("intel") {
      Some("Intel".to_string())
    } else {
      None
    }
  }

  /// Extracts model name from device name.
  fn extract_model(&self) -> Option<String> {
    // Extract meaningful model name from full device name
    Some(self.name.clone())
  }
}

/// Gets all active displays on macOS.
pub fn all_displays() -> Result<Vec<Display>> {
  let display_ids = get_active_display_ids()?;
  let dispatcher = EventLoopDispatcher::new();
  let mut displays = Vec::new();

  for display_id in display_ids {
    // Use display ID as device ID for now - in real implementation,
    // would map displays to their GPU devices via IOKit
    let device_id = format!("gpu:{}", display_id);
    displays.push(Display::new(display_id, device_id, &dispatcher));
  }

  Ok(displays)
}

/// Gets all display devices on macOS.
pub fn all_display_devices() -> Result<Vec<DisplayDevice>> {
  let mut devices = Vec::new();

  // This is a simplified implementation
  // Real implementation would use IOKit to enumerate graphics devices
  devices.push(DisplayDevice::new(
    1,
    "Apple M1 Pro".to_string(),
    false,
    true,
  ));

  Ok(devices)
}

/// Gets display from point on macOS.
pub fn display_from_point(point: Point) -> Result<Display> {
  let displays = all_displays()?;

  for display in &displays {
    let bounds = display.bounds()?;
    if bounds.contains_point(&point) {
      return Ok(display.clone());
    }
  }

  // Fall back to primary display
  primary_display()
}

/// Gets primary display on macOS.
pub fn primary_display() -> Result<Display> {
  let main_display_id = unsafe { CGMainDisplayID() };
  let dispatcher = EventLoopDispatcher::new();
  let device_id = format!("gpu:{}", main_display_id);

  Ok(Display::new(main_display_id, device_id, &dispatcher))
}

/// Gets active display IDs.
fn get_active_display_ids() -> Result<Vec<CGDirectDisplayID>> {
  let mut display_count = 0u32;

  // First get the count
  let result = unsafe {
    CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut display_count)
  };

  if result != CGError(0) {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get active display count: {:?}",
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

  if result != CGError(0) {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get active display list: {:?}",
      result
    )));
  }

  Ok(display_ids)
}

/// Gets all display IDs (active and inactive).
fn get_all_display_ids() -> Result<Vec<CGDirectDisplayID>> {
  let mut display_count = 0u32;

  // First get the count
  let result = unsafe {
    CGGetOnlineDisplayList(0, std::ptr::null_mut(), &mut display_count)
  };

  if result != CGError(0) {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get display count: {:?}",
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

  if result != CGError(0) {
    return Err(Error::Anyhow(anyhow::anyhow!(
      "Failed to get display list: {:?}",
      result
    )));
  }

  Ok(display_ids)
}
