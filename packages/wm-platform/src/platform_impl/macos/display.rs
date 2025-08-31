use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::NSScreen;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode,
  CGDisplayMirrorsDisplay, CGDisplayMode, CGDisplayRotation, CGError,
  CGGetActiveDisplayList, CGGetOnlineDisplayList, CGMainDisplayID,
};
use objc2_foundation::{ns_string, NSNumber};
use wm_common::{Point, Rect};

use crate::{
  platform_impl::MainThreadRef, ConnectionState, Dispatcher,
  DisplayDeviceId, DisplayId, MirroringState,
};

/// macOS-specific extensions for `Display`.
pub trait DisplayExtMacOs {
  /// Gets the Core Graphics display ID.
  fn cg_display_id(&self) -> CGDirectDisplayID;

  /// Gets the `NSScreen` instance for this display.
  ///
  /// `NSScreen` is always available for active displays. This method
  /// provides thread-safe access to the `NSScreen`.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ns_screen(&self) -> &MainThreadRef<Retained<NSScreen>>;
}

/// macOS-specific extensions for `DisplayDevice`.
pub trait DisplayDeviceExtMacOs {
  /// Gets the Core Graphics display ID.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn cg_display_id(&self) -> CGDirectDisplayID;
}

impl DisplayExtMacOs for crate::Display {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }

  fn ns_screen(&self) -> &MainThreadRef<Retained<NSScreen>> {
    self.inner.ns_screen()
  }
}

impl DisplayDeviceExtMacOs for crate::DisplayDevice {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }
}

/// macOS-specific display implementation.
#[derive(Clone, Debug)]
pub struct Display {
  pub(crate) cg_display_id: CGDirectDisplayID,
  pub(crate) ns_screen: MainThreadRef<Retained<NSScreen>>,
}

impl Display {
  /// Creates a new macOS display.
  #[must_use]
  pub fn new(
    ns_screen: MainThreadRef<Retained<NSScreen>>,
  ) -> crate::Result<Self> {
    let cg_display_id = ns_screen
      .with(|screen| {
        let device_description = screen.deviceDescription();

        device_description
          .objectForKey(ns_string!("NSScreenNumber"))
          .and_then(|val| {
            val.downcast_ref::<NSNumber>().map(NSNumber::as_u32)
          })
      })?
      .ok_or(crate::Error::DisplayNotFound)?;

    Ok(Self {
      cg_display_id,
      ns_screen,
    })
  }

  /// Gets the [`crate::Display`] ID.
  pub fn id(&self) -> DisplayId {
    DisplayId(self.cg_display_id)
  }

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }

  /// Gets the `NSScreen` instance for this display.
  pub fn ns_screen(&self) -> &MainThreadRef<Retained<NSScreen>> {
    &self.ns_screen
  }

  /// Gets the display name.
  pub fn name(&self) -> crate::Result<String> {
    self.ns_screen.with(|screen| {
      let name = unsafe { screen.localizedName() };
      Ok(name.to_string())
    })?
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> crate::Result<Rect> {
    let cg_rect = unsafe { CGDisplayBounds(self.cg_display_id) };

    #[allow(clippy::cast_possible_truncation)]
    Ok(Rect::from_ltrb(
      cg_rect.origin.x as i32,
      cg_rect.origin.y as i32,
      (cg_rect.origin.x + cg_rect.size.width) as i32,
      (cg_rect.origin.y + cg_rect.size.height) as i32,
    ))
  }

  /// Gets the working area rectangle (excluding dock and menu bar).
  pub fn working_area(&self) -> crate::Result<Rect> {
    self.ns_screen.with(|screen| {
      let visible_frame = screen.visibleFrame();

      #[allow(clippy::cast_possible_truncation)]
      Ok(Rect::from_ltrb(
        visible_frame.origin.x as i32,
        (visible_frame.origin.y + visible_frame.size.height) as i32,
        (visible_frame.origin.x + visible_frame.size.width) as i32,
        visible_frame.origin.y as i32,
      ))
    })?
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> crate::Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    self
      .ns_screen
      .with(|screen| screen.backingScaleFactor() as f32)
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> crate::Result<u32> {
    let scale_factor = self.scale_factor()?;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((72.0 * scale_factor) as u32)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> crate::Result<bool> {
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.cg_display_id == main_display_id)
  }

  /// Gets the display devices for this display.
  pub fn devices(&self) -> crate::Result<Vec<crate::DisplayDevice>> {
    // TODO: Get main device as well as any devices that are mirroring this
    // display.
    let device = DisplayDevice::new(self.cg_display_id);
    Ok(vec![device.into()])
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(&self) -> crate::Result<crate::DisplayDevice> {
    self
      .devices()?
      .into_iter()
      .find(|device| {
        matches!(
          device.mirroring_state(),
          Ok(None | Some(MirroringState::Source))
        )
      })
      .ok_or(crate::Error::DisplayNotFound)
  }
}

impl From<Display> for crate::Display {
  fn from(display: Display) -> Self {
    crate::Display { inner: display }
  }
}

impl PartialEq for Display {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Display {}

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
    // For now, use CGDirectDisplayID directly as a u32
    // TODO: Implement proper CFUUID support using
    // CGDisplayCreateUUIDFromDisplayID
    DisplayDeviceId(self.cg_display_id)
  }

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> crate::Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(unsafe { CGDisplayRotation(self.cg_display_id) } as f32)
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> crate::Result<ConnectionState> {
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) };

    // TODO: Implement this properly.
    if display_mode.is_none() {
      Ok(ConnectionState::Disconnected)
    } else {
      Ok(ConnectionState::Active)
    }
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> crate::Result<f32> {
    // Calling `CGDisplayModeRelease` on the display mode is not needed as
    // it's functionally equivalent to `CFRelease`.
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) }
        .ok_or(crate::Error::DisplayModeNotFound)?;

    let refresh_rate =
      unsafe { CGDisplayMode::refresh_rate(Some(&display_mode)) };

    #[allow(clippy::cast_possible_truncation)]
    Ok(refresh_rate as f32)
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> crate::Result<bool> {
    // TODO: Implement this properly.
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.cg_display_id == main_display_id)
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> crate::Result<Option<MirroringState>> {
    let mirrored_display =
      unsafe { CGDisplayMirrorsDisplay(self.cg_display_id) };

    if mirrored_display == 0 {
      // This display is not mirroring another display
      // Check if another display is mirroring this one by querying active
      // displays
      let mut displays: Vec<CGDirectDisplayID> = vec![0; 32];
      let mut display_count: u32 = 0;

      let result = unsafe {
        CGGetActiveDisplayList(
          displays.len() as u32,
          displays.as_mut_ptr(),
          &mut display_count,
        )
      };

      if result == CGError::Success {
        displays.truncate(display_count as usize);
        for &display_id in &displays {
          if display_id == self.cg_display_id {
            continue; // Skip self
          }
          let other_mirrored =
            unsafe { CGDisplayMirrorsDisplay(display_id) };
          if other_mirrored == self.cg_display_id {
            // Another display is mirroring this one, so this is the source
            return Ok(Some(MirroringState::Source));
          }
        }
      }
      Ok(None)
    } else {
      // This display is mirroring another display, so it's a target
      Ok(Some(MirroringState::Target))
    }
  }
}

impl From<DisplayDevice> for crate::DisplayDevice {
  fn from(device: DisplayDevice) -> Self {
    crate::DisplayDevice { inner: device }
  }
}

/// Gets all active displays on macOS.
///
/// Must be called on the main thread.
pub fn all_displays(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::Display>> {
  let dispatcher_clone = dispatcher.clone();
  dispatcher.dispatch_sync(move || {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    let mut displays = Vec::new();

    for screen in NSScreen::screens(mtm) {
      let ns_screen = MainThreadRef::new(dispatcher_clone.clone(), screen);
      displays.push(Display::new(ns_screen)?.into());
    }

    Ok(displays)
  })?
}

/// Gets all display devices on macOS.
pub fn all_display_devices(
  _dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::DisplayDevice>> {
  let mut displays: Vec<CGDirectDisplayID> = vec![0; 32]; // Max 32 displays
  let mut display_count: u32 = 0;

  let result = unsafe {
    CGGetOnlineDisplayList(
      displays.len() as u32,
      displays.as_mut_ptr(),
      &mut display_count,
    )
  };

  if result != CGError::Success {
    return Err(crate::Error::DisplayEnumerationFailed);
  }

  displays.truncate(display_count as usize);

  Ok(
    displays
      .into_iter()
      .map(DisplayDevice::new)
      .map(Into::into)
      .collect(),
  )
}

/// Gets active display devices on macOS.
pub fn active_display_devices(
  _dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::DisplayDevice>> {
  let mut displays: Vec<CGDirectDisplayID> = vec![0; 32]; // Max 32 displays
  let mut display_count: u32 = 0;

  let result = unsafe {
    CGGetActiveDisplayList(
      displays.len() as u32,
      displays.as_mut_ptr(),
      &mut display_count,
    )
  };

  if result != CGError::Success {
    return Err(crate::Error::DisplayEnumerationFailed);
  }

  displays.truncate(display_count as usize);

  Ok(
    displays
      .into_iter()
      .map(DisplayDevice::new)
      .map(Into::into)
      .collect(),
  )
}

/// Gets display from point.
pub fn display_from_point(
  point: Point,
  dispatcher: &Dispatcher,
) -> crate::Result<crate::Display> {
  let displays = all_displays(dispatcher)?;

  for display in displays {
    let bounds = display.bounds()?;
    if bounds.contains_point(&point) {
      return Ok(display);
    }
  }

  Err(crate::Error::DisplayNotFound)
}

/// Gets primary display on macOS.
pub fn primary_display(
  dispatcher: &Dispatcher,
) -> crate::Result<crate::Display> {
  let displays = all_displays(dispatcher)?;

  for display in displays {
    if display.is_primary()? {
      return Ok(display);
    }
  }

  Err(crate::Error::PrimaryDisplayNotFound)
}
