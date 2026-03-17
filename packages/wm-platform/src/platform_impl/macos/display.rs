use std::sync::Arc;

use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::NSScreen;
use objc2_core_foundation::{CFRetained, CFUUID};
use objc2_core_graphics::{
  CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode,
  CGDisplayMirrorsDisplay, CGDisplayMode, CGDisplayRotation, CGError,
  CGGetActiveDisplayList, CGGetOnlineDisplayList, CGMainDisplayID,
};
use objc2_foundation::{ns_string, NSNumber, NSRect};

use crate::{
  platform_impl::ffi, ConnectionState, Dispatcher, DisplayDeviceId,
  DisplayId, MirroringState, Point, Rect, ThreadBound,
};

/// Platform-specific implementation of [`Display`].
#[derive(Clone, Debug)]
pub(crate) struct Display {
  cg_display_id: CGDirectDisplayID,
  ns_screen: Arc<ThreadBound<Retained<NSScreen>>>,
}

impl Display {
  /// Creates an instance of `Display`.
  pub(crate) fn new(
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

  /// Implements [`Display::id`].
  pub(crate) fn id(&self) -> DisplayId {
    DisplayId(self.cg_display_id)
  }

  /// Implements [`Display::name`].
  pub(crate) fn name(&self) -> crate::Result<String> {
    self.ns_screen.with(|screen| {
      let name = screen.localizedName();
      Ok(name.to_string())
    })?
  }

  /// Implements [`Display::bounds`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn bounds(&self) -> crate::Result<Rect> {
    let cg_rect = CGDisplayBounds(self.cg_display_id);

    #[allow(clippy::cast_possible_truncation)]
    Ok(Rect::from_xy(
      cg_rect.origin.x as i32,
      cg_rect.origin.y as i32,
      cg_rect.size.width as i32,
      cg_rect.size.height as i32,
    ))
  }

  /// Implements [`Display::working_area`].
  pub(crate) fn working_area(&self) -> crate::Result<Rect> {
    let primary_display_bounds = {
      let bounds = CGDisplayBounds(CGMainDisplayID());

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

  /// Implements [`Display::scale_factor`].
  pub(crate) fn scale_factor(&self) -> crate::Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    self
      .ns_screen
      .with(|screen| screen.backingScaleFactor() as f32)
  }

  /// Implements [`Display::dpi`].
  pub(crate) fn dpi(&self) -> crate::Result<u32> {
    let scale_factor = self.scale_factor()?;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((72.0 * scale_factor) as u32)
  }

  /// Implements [`Display::is_primary`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_primary(&self) -> crate::Result<bool> {
    let main_display_id = CGMainDisplayID();
    Ok(self.cg_display_id == main_display_id)
  }

  /// Implements [`Display::devices`].
  pub(crate) fn devices(
    &self,
  ) -> crate::Result<Vec<crate::DisplayDevice>> {
    let main_device = DisplayDevice::new(
      self.cg_display_id,
      cg_display_uuid(self.cg_display_id)?,
    );

    // TODO: Get devices that are mirroring this display as well.
    Ok(vec![main_device.into()])
  }

  /// Implements [`Display::main_device`].
  pub(crate) fn main_device(&self) -> crate::Result<crate::DisplayDevice> {
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

  /// Implements [`DisplayExtMacOs::cg_display_id`].
  pub(crate) fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }

  /// Implements [`DisplayExtMacOs::ns_screen`].
  pub(crate) fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>> {
    &self.ns_screen
  }
}

/// Transforms an AppKit screen rectangle (e.g. `NSScreen.visibleFrame`)
/// into Core Graphics coordinate space (e.g. `CGDisplayBounds`).
///
/// AppKit has (0,0) at the bottom-left corner of the primary display,
/// whereas Core Graphics has it at the top-left corner. So we can convert
/// between the two by offsetting the Y-axis by the primary display's
/// height.
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

/// Platform-specific implementation of [`DisplayDevice`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DisplayDevice {
  cg_display_id: CGDirectDisplayID,
  uuid: CFRetained<CFUUID>,
}

impl DisplayDevice {
  /// Creates an instance of `DisplayDevice`.
  #[must_use]
  pub(crate) fn new(
    cg_display_id: CGDirectDisplayID,
    uuid: CFRetained<CFUUID>,
  ) -> Self {
    Self {
      cg_display_id,
      uuid,
    }
  }

  /// Implements [`DisplayDevice::id`].
  pub(crate) fn id(&self) -> DisplayDeviceId {
    // SAFETY: Can assume that the `CFUUID` is valid regardless of whether
    // the underlying display device is still alive.
    let uuid_string = CFUUID::new_string(None, Some(&self.uuid))
      .unwrap()
      .to_string();

    DisplayDeviceId(uuid_string)
  }

  /// Implements [`DisplayDevice::rotation`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn rotation(&self) -> crate::Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(CGDisplayRotation(self.cg_display_id) as f32)
  }

  /// Implements [`DisplayDevice::refresh_rate`].
  pub(crate) fn refresh_rate(&self) -> crate::Result<f32> {
    // NOTE: Calling `CGDisplayModeRelease` on cleanup is not needed, since
    // it's equivalent to `CFRelease` in this case. Ref: https://developer.apple.com/documentation/coregraphics/cgdisplaymoderelease
    let display_mode = CGDisplayCopyDisplayMode(self.cg_display_id)
      .ok_or(crate::Error::DisplayModeNotFound)?;

    let refresh_rate = CGDisplayMode::refresh_rate(Some(&display_mode));

    #[allow(clippy::cast_possible_truncation)]
    Ok(refresh_rate as f32)
  }

  /// Implements [`DisplayDevice::is_builtin`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_builtin(&self) -> crate::Result<bool> {
    // TODO: Implement this properly.
    let main_display_id = CGMainDisplayID();
    Ok(self.cg_display_id == main_display_id)
  }

  /// Implements [`DisplayDevice::connection_state`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn connection_state(&self) -> crate::Result<ConnectionState> {
    let display_mode = CGDisplayCopyDisplayMode(self.cg_display_id);

    // TODO: Implement this properly.
    if display_mode.is_none() {
      Ok(ConnectionState::Disconnected)
    } else {
      Ok(ConnectionState::Active)
    }
  }

  /// Implements [`DisplayDevice::mirroring_state`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn mirroring_state(
    &self,
  ) -> crate::Result<Option<MirroringState>> {
    let mirrored_display = CGDisplayMirrorsDisplay(self.cg_display_id);

    // TODO: Clean this up.
    if mirrored_display == 0 {
      // This display is not mirroring another display
      // Check if another display is mirroring this one by querying active
      // displays
      let mut displays: Vec<CGDirectDisplayID> = vec![0; 32];
      let mut display_count: u32 = 0;

      #[allow(clippy::cast_possible_truncation)]
      let result = unsafe {
        CGGetActiveDisplayList(
          displays.len() as u32,
          displays.as_mut_ptr(),
          &raw mut display_count,
        )
      };

      if result == CGError::Success {
        displays.truncate(display_count as usize);
        for &display_id in &displays {
          if display_id == self.cg_display_id {
            continue; // Skip self
          }
          let other_mirrored = CGDisplayMirrorsDisplay(display_id);
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

  /// Implements [`DisplayDeviceExtMacOs::cg_display_id`].
  pub(crate) fn cg_display_id(&self) -> CGDirectDisplayID {
    self.cg_display_id
  }
}

impl From<DisplayDevice> for crate::DisplayDevice {
  fn from(device: DisplayDevice) -> Self {
    crate::DisplayDevice { inner: device }
  }
}

/// Gets the UUID for a display device from its `CGDirectDisplayID`.
///
/// This UUID is stable across reboots, whereas `CGDirectDisplayID` is not.
fn cg_display_uuid(
  cg_display_id: CGDirectDisplayID,
) -> crate::Result<CFRetained<CFUUID>> {
  let ptr =
    unsafe { ffi::CGDisplayCreateUUIDFromDisplayID(cg_display_id) };

  ptr.map(|ptr| unsafe { CFRetained::from_raw(ptr) }).ok_or(
    crate::Error::InvalidPointer(
      "Failed to create UUID for display device".to_string(),
    ),
  )
}

/// Implements [`Dispatcher::displays`].
pub(crate) fn all_displays(
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

/// Implements [`Dispatcher::display_devices`].
pub(crate) fn all_display_devices(
  _: &Dispatcher,
) -> crate::Result<Vec<crate::DisplayDevice>> {
  let mut cg_display_ids: Vec<CGDirectDisplayID> = vec![0; 32]; // Max 32 displays
  let mut display_count: u32 = 0;

  #[allow(clippy::cast_possible_truncation)]
  let result = unsafe {
    CGGetOnlineDisplayList(
      cg_display_ids.len() as u32,
      cg_display_ids.as_mut_ptr(),
      &raw mut display_count,
    )
  };

  if result != CGError::Success {
    return Err(crate::Error::DisplayEnumerationFailed);
  }

  cg_display_ids.truncate(display_count as usize);

  cg_display_ids
    .into_iter()
    .map(|cg_display_id| {
      Ok(
        DisplayDevice::new(cg_display_id, cg_display_uuid(cg_display_id)?)
          .into(),
      )
    })
    .collect()
}

/// Implements [`Dispatcher::display_from_point`].
pub(crate) fn display_from_point(
  point: &Point,
  dispatcher: &Dispatcher,
) -> crate::Result<crate::Display> {
  let displays = all_displays(dispatcher)?;

  for display in displays {
    let bounds = display.bounds()?;
    if bounds.contains_point(point) {
      return Ok(display);
    }
  }

  Err(crate::Error::DisplayNotFound)
}

/// Implements [`Dispatcher::primary_display`].
pub(crate) fn primary_display(
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

/// Implements [`Dispatcher::nearest_display`].
///
/// NOTE: This was benchmarked to be 400-600µs on initial retrieval and
/// 150-300µs on subsequent retrievals. Using `CGGetDisplaysWithRect` and
/// getting the corresponding `NSScreen` was found to be slightly slower
/// (700-800µs and then 200-300µs on subsequent retrievals).
pub(crate) fn nearest_display(
  native_window: &crate::NativeWindow,
  dispatcher: &Dispatcher,
) -> crate::Result<crate::Display> {
  dispatcher.dispatch_sync(|| {
    // Get the window's frame in screen coordinates.
    let window_frame = native_window.frame()?;

    let screens = all_displays(dispatcher)?;
    let mut best_screen = None;
    let mut max_intersection_area = 0;

    // TODO: Clean this up.
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
