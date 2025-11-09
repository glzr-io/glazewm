use std::sync::Arc;

use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::NSScreen;
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode,
  CGDisplayMirrorsDisplay, CGDisplayMode, CGDisplayRotation, CGError,
  CGGetActiveDisplayList, CGGetOnlineDisplayList, CGMainDisplayID,
};
use objc2_foundation::{ns_string, NSNumber, NSRect};

use crate::{
  ConnectionState, Dispatcher, DisplayDeviceId, DisplayId, MirroringState,
  Point, Rect, ThreadBound,
};

/// macOS-specific extensions for [`Display`].
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
  fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>>;
}

impl DisplayExtMacOs for crate::Display {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }

  fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>> {
    self.inner.ns_screen()
  }
}

/// macOS-specific extensions for [`DisplayDevice`].
pub trait DisplayDeviceExtMacOs {
  /// Gets the Core Graphics display ID.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn cg_display_id(&self) -> CGDirectDisplayID;
}

impl DisplayDeviceExtMacOs for crate::DisplayDevice {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }
}

/// macOS-specific implementation of [`Display`].
#[derive(Clone, Debug)]
pub struct Display {
  cg_display_id: CGDirectDisplayID,
  ns_screen: Arc<ThreadBound<Retained<NSScreen>>>,
}

impl Display {
  /// macOS-specific implementation of [`Display::new`].
  #[must_use]
  pub fn new(
    ns_screen: ThreadBound<Retained<NSScreen>>,
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
      ns_screen: Arc::new(ns_screen),
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
  pub fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>> {
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
    Ok(Rect::from_xy(
      cg_rect.origin.x as i32,
      cg_rect.origin.y as i32,
      cg_rect.size.width as i32,
      cg_rect.size.height as i32,
    ))
  }

  /// Gets the working area rectangle (excludes dock and menu bar).
  pub fn working_area(&self) -> crate::Result<Rect> {
    let primary_display_bounds = {
      let bounds = unsafe { CGDisplayBounds(CGMainDisplayID()) };

      #[allow(clippy::cast_possible_truncation)]
      Rect::from_xy(
        bounds.origin.x as i32,
        bounds.origin.y as i32,
        bounds.size.width as i32,
        bounds.size.height as i32,
      )
    };

    self.ns_screen.with(|screen| {
      // Convert `NSScreen.visibleFrame` into the same coordinate space as
      // `CGDisplayBounds`.
      Ok(appkit_rect_to_cg_rect(
        screen.visibleFrame(),
        &primary_display_bounds,
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

/// Transforms an AppKit screen rectangle (e.g. `NSScreen.visibleFrame`)
/// into Core Graphics coordinate space (e.g. `CGDisplayBounds`).
///
/// AppKit has (0,0) at the bottom-left corner of the primary display,
/// whereas Core Graphics has it at the top-left corner. So we can convert
/// between the two by offsetting the Y-axis by the primary display height.
///
/// # Arguments
/// * `appkit_rect` - The rectangle in AppKit coordinate space.
/// * `primary_display_bounds` - The bounds of the primary display in CG
///   space.
fn appkit_rect_to_cg_rect(
  appkit_rect: NSRect,
  primary_display_bounds: &Rect,
) -> Rect {
  let adjusted_y = f64::from(primary_display_bounds.height())
    - (appkit_rect.origin.y + appkit_rect.size.height);

  #[allow(clippy::cast_possible_truncation)]
  Rect::from_xy(
    appkit_rect.origin.x as i32,
    adjusted_y as i32,
    appkit_rect.size.width as i32,
    appkit_rect.size.height as i32,
  )
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

/// macOS-specific implementation of [`DisplayDevice`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) cg_display_id: CGDirectDisplayID,
}

impl DisplayDevice {
  /// macOS-specific implementation of [`DisplayDevice::new`].
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
  dispatcher.dispatch_sync(|| {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    let mut displays = Vec::new();

    for screen in NSScreen::screens(mtm) {
      let ns_screen = ThreadBound::new(screen, dispatcher.clone());
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
      &raw mut display_count,
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
  dispatcher: &Dispatcher,
  point: Point,
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
  dispatcher.dispatch_sync(|| {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    let ns_screen = ThreadBound::new(
      NSScreen::mainScreen(mtm).ok_or(crate::Error::DisplayNotFound)?,
      dispatcher.clone(),
    );

    Display::new(ns_screen).map(Into::into)
  })?
}

/// Gets the nearest display to a window on macOS.
///
/// Returns the display that contains the largest area of the window's
/// frame. If the window is completely off-screen, returns the main
/// display.
///
/// NOTE: This was benchmarked to be 400-600µs on initial retrieval and
/// 150-300µs on subsequent retrievals. Instead using
/// `CGGetDisplaysWithRect` (and getting the corresponding `NSScreen`)
/// was found to be slightly slower (700-800µs and 200-300µs respectively).
pub fn nearest_display(
  native_window: &crate::NativeWindow,
  dispatcher: &Dispatcher,
) -> crate::Result<crate::Display> {
  dispatcher.dispatch_sync(|| {
    // Get the window's frame in screen coordinates.
    let window_frame = native_window.frame()?;

    let screens = all_displays(dispatcher)?;
    let mut best_screen = None;
    let mut max_intersection_area = 0;

    // Iterate through all screens to find the one with the largest
    // intersection with the window.
    for screen in screens {
      let screen_frame = screen.bounds()?;

      // Calculate intersection area.
      let intersection_x = i32::max(window_frame.x(), screen_frame.x());
      let intersection_y = i32::max(window_frame.y(), screen_frame.y());
      let intersection_width = i32::min(
        window_frame.x() + window_frame.width(),
        screen_frame.x() + screen_frame.width(),
      ) - intersection_x;
      let intersection_height = i32::min(
        window_frame.y() + window_frame.height(),
        screen_frame.y() + screen_frame.height(),
      ) - intersection_y;

      // If there's a valid intersection, calculate its area.
      if intersection_width > 0 && intersection_height > 0 {
        let area = intersection_width * intersection_height;
        if area > max_intersection_area {
          max_intersection_area = area;
          best_screen = Some(screen);
        }
      }
    }

    // If we found a screen with intersection, use it. Otherwise, if the
    // window is off-screen, use the main screen.
    best_screen
      .or_else(|| primary_display(dispatcher).ok())
      .ok_or(crate::Error::DisplayNotFound)
  })?
}
