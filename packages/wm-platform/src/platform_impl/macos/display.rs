use objc2_app_kit::NSScreen;
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode,
  CGDisplayMirrorsDisplay, CGDisplayMode, CGDisplayModeGetPixelHeight,
  CGDisplayModeGetPixelWidth, CGError, CGGetActiveDisplayList,
  CGGetOnlineDisplayList, CGMainDisplayID,
};
use wm_common::{Point, Rect};

use crate::{
  display::{ConnectionState, DisplayDeviceId, DisplayId, MirroringState},
  error::{Error, Result},
  platform_impl::MainThreadRef,
};

/// macOS-specific display implementation.
/// TODO: Add `PartialEq` and `Eq`.
#[derive(Clone, Debug)]
pub struct Display {
  pub(crate) display_id: CGDirectDisplayID,
  pub(crate) ns_screen: MainThreadRef<CFRetained<NSScreen>>,
}

impl Display {
  /// Creates a new macOS display.
  #[must_use]
  pub fn new(
    display_id: CGDirectDisplayID,
    ns_screen: MainThreadRef<CFRetained<NSScreen>>,
  ) -> Self {
    Self {
      display_id,
      ns_screen,
    }
  }

  /// Gets the [`crate::Display`] ID.
  pub fn id(&self) -> DisplayId {
    DisplayId(self.display_id)
  }

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.display_id
  }

  /// Gets the `NSScreen` instance for this display.
  pub fn ns_screen(&self) -> &MainThreadRef<CFRetained<NSScreen>> {
    &self.ns_screen
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    self.ns_screen.with(|screen| {
      let name = unsafe { screen.localizedName() };
      name.to_string()
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
    self.ns_screen.with(|screen| {
      let visible_frame = screen.visibleFrame();
      #[allow(clippy::cast_possible_truncation)]
      Ok(Rect::from_ltrb(
        visible_frame.origin.x as i32,
        (visible_frame.origin.y + visible_frame.size.height) as i32,
        (visible_frame.origin.x + visible_frame.size.width) as i32,
        visible_frame.origin.y as i32,
      ))
    })
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> Result<f32> {
    self.ns_screen.with(|screen| {
      #[allow(clippy::cast_possible_truncation)]
      Ok(screen.backingScaleFactor() as f32)
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
    // TODO
    Ok(24)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    // TODO: Is this correct?
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.display_id == main_display_id)
  }

  /// Gets the display devices for this display.
  pub fn devices(&self) -> Result<Vec<crate::display::DisplayDevice>> {
    let all_devices = all_display_devices()?;

    // TODO
    Ok(
      all_devices
        .into_iter()
        .map(crate::display::DisplayDevice::from_platform_impl)
        .collect(),
    )
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(
    &self,
  ) -> Result<Option<crate::display::DisplayDevice>> {
    // TODO
    let devices = self.devices()?;

    // Find first device that is not mirroring.
    for device in devices {
      if device.mirroring_state()? == MirroringState::None
        || device.mirroring_state()? == MirroringState::Source
      {
        return Ok(Some(device));
      }
    }

    Ok(None)
  }

  /// Checks if this is a built-in display.
  pub fn is_builtin(&self) -> Result<bool> {
    // TODO
    self.is_primary()
  }
}

/// macOS-specific display device implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) cg_display_id: CGDirectDisplayID,
}

impl DisplayDevice {
  /// Creates a new macOS display device.
  #[must_use]
  pub fn new(cg_display_id: CGDirectDisplayID) -> Self {
    Self { cg_display_id }
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    DisplayDeviceId(self.cg_display_id)
  }

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }

  /// Gets the device name.
  pub fn name(&self) -> Result<String> {
    // TODO
    Ok(format!("Display Device {}", self.cg_display_id))
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    // TODO
    Ok(0.0)
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
    // TODO
    Ok(ConnectionState::Active)
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) };

    let refresh_rate =
      unsafe { CGDisplayMode::refresh_rate(display_mode.as_deref()) };

    Ok(refresh_rate as f32)
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> Result<bool> {
    // TODO
    Ok(false)
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<MirroringState> {
    let mirrored_display =
      unsafe { CGDisplayMirrorsDisplay(self.cg_display_id) };

    // TODO
    if mirrored_display == 0 {
      Ok(MirroringState::None)
    } else {
      Ok(MirroringState::Target)
    }
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
}

/// Gets all active displays on macOS.
pub fn all_displays() -> Result<Vec<Display>> {
  let display_ids = get_active_display_ids()?;

  let mut displays = Vec::new();

  for display_id in display_ids {
    // TODO
    displays.push(Display::new(display_id, ns_screen));
  }

  Ok(displays)
}

/// Gets all display devices on macOS.
pub fn all_display_devices() -> Result<Vec<DisplayDevice>> {
  let display_ids = get_all_display_ids()?;
  let mut devices = Vec::new();

  for display_id in display_ids {
    devices.push(DisplayDevice::new(display_id));
  }

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
  let _main_display_id = unsafe { CGMainDisplayID() };

  // TODO
  Err(Error::Anyhow(anyhow::anyhow!(
    "Primary display not implemented yet"
  )))
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

// TODO: Implement proper MainThreadRef creation when needed
// This would require proper integration with the event loop context
