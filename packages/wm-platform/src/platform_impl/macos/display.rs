use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::NSScreen;
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode,
  CGDisplayMirrorsDisplay, CGDisplayMode, CGError, CGGetActiveDisplayList,
  CGGetOnlineDisplayList, CGMainDisplayID,
};
use wm_common::{Point, Rect};

use crate::{
  display::{ConnectionState, DisplayDeviceId, DisplayId, MirroringState},
  error::Result,
  platform_impl::{EventLoopDispatcher, MainThreadRef},
};

/// macOS-specific extensions for `Display`.
pub trait DisplayExtMacos {
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
  fn ns_screen(&self) -> Result<MainThreadRef<CFRetained<NSScreen>>>;

  /// Checks if this is a built-in display.
  ///
  /// Returns `true` for embedded displays (like laptop screens).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn is_builtin(&self) -> Result<bool>;
}

/// macOS-specific extensions for `DisplayDevice`.
pub trait DisplayDeviceExtMacos {
  /// Gets the Core Graphics display ID.
  fn cg_display_id(&self) -> CGDirectDisplayID;
}

impl DisplayExtMacos for Display {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id()
  }

  fn ns_screen(&self) -> Result<MainThreadRef<CFRetained<NSScreen>>> {
    Ok(self.ns_screen().clone())
  }

  fn is_builtin(&self) -> Result<bool> {
    todo!()
  }
}

impl DisplayDeviceExtMacos for DisplayDevice {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id()
  }
}

/// macOS-specific display implementation.
/// TODO: Add `PartialEq` and `Eq`.
#[derive(Clone, Debug)]
pub struct Display {
  pub(crate) cg_display_id: CGDirectDisplayID,
  pub(crate) ns_screen: MainThreadRef<Retained<NSScreen>>,
}

impl Display {
  /// Creates a new macOS display.
  #[must_use]
  pub fn new(ns_screen: MainThreadRef<Retained<NSScreen>>) -> Self {
    let cg_display_id = unsafe { NSScreen::display_id(ns_screen) };

    Self {
      cg_display_id,
      ns_screen,
    }
  }

  /// Gets the [`crate::Display`] ID.
  pub fn id(&self) -> DisplayId {
    // TODO: Not sure what to use as ID here. The Core Graphics display ID
    // is used as a device ID and shouldn't also be used as a display ID.
    todo!()
  }

  /// Gets the Core Graphics display ID.
  pub fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }

  /// Gets the NSScreen instance for this display.
  pub fn ns_screen(&self) -> &MainThreadRef<Retained<NSScreen>> {
    &self.ns_screen
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    self.ns_screen.with(|screen| {
      let name = unsafe { screen.localizedName() };
      Ok(name.to_string())
    })?
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> Result<Rect> {
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
    })?
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

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.cg_display_id == main_display_id)
  }

  /// Gets the display devices for this display.
  pub fn devices(&self) -> Result<Vec<crate::display::DisplayDevice>> {
    // TODO: Get main device as well as any devices that are mirroring this
    // display.
    let device = DisplayDevice::new(self.cg_display_id);
    Ok(vec![crate::display::DisplayDevice::from_platform_impl(
      device,
    )])
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(
    &self,
  ) -> Result<Option<crate::display::DisplayDevice>> {
    let devices = self.devices()?;

    for device in devices {
      let mirroring_state = device.mirroring_state()?;
      if mirroring_state.is_none()
        || mirroring_state == Some(MirroringState::Source)
      {
        return Ok(Some(device));
      }
    }

    Ok(None)
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

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    // TODO: Implement this
    Ok(0.0)
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
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
  pub fn refresh_rate(&self) -> Result<f32> {
    let display_mode =
      unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) }
        .ok_or(crate::Error::DisplayModeNotFound)?;

    let refresh_rate =
      unsafe { CGDisplayMode::refresh_rate(Some(&display_mode)) };

    Ok(refresh_rate as f32)
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> Result<bool> {
    // TODO: Implement this properly.
    let main_display_id = unsafe { CGMainDisplayID() };
    Ok(self.cg_display_id == main_display_id)
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<Option<MirroringState>> {
    let mirrored_display =
      unsafe { CGDisplayMirrorsDisplay(self.cg_display_id) };

    // TODO: Get whether this is the source of mirroring.
    // Maybe CGDisplayIsInMirrorSet(self.platform_data.cg_display_id)
    // && CGDisplayIsAlwaysInMirrorSet(self.platform_data.cg_display_id)

    if mirrored_display == 0 {
      Ok(None)
    } else {
      Ok(Some(MirroringState::Target))
    }
  }
}

/// Gets all active displays on macOS.
///
/// Must be called on the main thread.
pub fn all_displays(
  dispatcher: EventLoopDispatcher,
) -> Result<Vec<Display>> {
  let mtm =
    MainThreadMarker::new().ok_or(crate::Error::NotMainThread.into())?;

  let mut displays = Vec::new();

  for screen in unsafe { NSScreen::screens(mtm) } {
    let ns_screen = MainThreadRef::new(dispatcher.clone(), screen);
    displays.push(Display::new(ns_screen));
  }

  Ok(displays)
}

/// Gets all display devices on macOS.
pub fn all_display_devices(
  dispatcher: EventLoopDispatcher,
) -> Result<Vec<DisplayDevice>> {
  todo!()
}

/// Gets active display devices on macOS.
pub fn active_display_devices(
  dispatcher: EventLoopDispatcher,
) -> Result<Vec<DisplayDevice>> {
  todo!()
}

/// Gets display from point.
pub fn display_from_point(
  point: Point,
  dispatcher: EventLoopDispatcher,
) -> Result<Display> {
  todo!()
}

/// Gets primary display on macOS.
pub fn primary_display(
  dispatcher: EventLoopDispatcher,
) -> Result<Display> {
  todo!()
}
