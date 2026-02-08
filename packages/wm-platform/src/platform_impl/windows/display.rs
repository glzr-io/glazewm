use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, LPARAM, POINT, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW,
      GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, DEVMODEW,
      DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, ENUM_CURRENT_SETTINGS, HDC,
      HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
      MONITOR_DEFAULTTOPRIMARY,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};

use crate::{
  display::{
    ConnectionState, DisplayDeviceId, DisplayId, MirroringState,
    OutputTechnology,
  },
  platform_impl::NativeWindowWindowsExt,
  Dispatcher, NativeWindow, Point, Rect,
};

/// Windows-specific extensions for `Display`.
pub trait DisplayExtWindows {
  /// Gets the monitor handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hmonitor(&self) -> HMONITOR;
}

/// Windows-specific extensions for `DisplayDevice`.
pub trait DisplayDeviceExtWindows {
  /// Gets the device interface path.
  ///
  /// This can be an empty string for virtual display devices.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn device_interface_path(&self) -> &str;

  /// Gets the hardware ID from the device interface path.
  ///
  /// # Example usage
  ///
  /// ```rust,no_run
  /// device.device_interface_path(); // "\\?\DISPLAY#DEL40A3#5&1234abcd&0&UID256#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}"
  /// device.hardware_id(); // Some("DEL40A3")
  /// ```
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hardware_id(&self) -> Option<String>;

  /// Gets the output technology.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn output_technology(&self) -> crate::Result<Option<OutputTechnology>>;
}

impl DisplayExtWindows for crate::Display {
  fn hmonitor(&self) -> HMONITOR {
    HMONITOR(self.inner.monitor_handle)
  }
}

impl DisplayDeviceExtWindows for crate::DisplayDevice {
  fn device_interface_path(&self) -> &str {
    &self.inner.device_interface_path
  }

  fn hardware_id(&self) -> Option<String> {
    self.inner.hardware_id()
  }

  fn output_technology(&self) -> crate::Result<Option<OutputTechnology>> {
    self.inner.output_technology()
  }
}

/// Windows-specific implementation of [`Display`].
#[derive(Clone, Debug)]
pub struct Display {
  pub(crate) monitor_handle: isize,
}

impl Display {
  /// Creates a new Windows display from monitor handle.
  #[must_use]
  pub fn new(monitor_handle: isize) -> Self {
    Self { monitor_handle }
  }

  /// Gets the unique identifier for this display.
  pub fn id(&self) -> DisplayId {
    DisplayId(self.monitor_handle)
  }

  /// Gets the display name.
  pub fn name(&self) -> crate::Result<String> {
    Ok(
      String::from_utf16_lossy(&self.monitor_info_ex()?.szDevice)
        .trim_end_matches('\0')
        .to_string(),
    )
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> crate::Result<Rect> {
    let rc = self.monitor_info_ex()?.monitorInfo.rcMonitor;
    Ok(Rect::from_ltrb(rc.left, rc.top, rc.right, rc.bottom))
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> crate::Result<Rect> {
    let rc = self.monitor_info_ex()?.monitorInfo.rcWork;
    Ok(Rect::from_ltrb(rc.left, rc.top, rc.right, rc.bottom))
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> crate::Result<f32> {
    let dpi = self.dpi()?;
    #[allow(clippy::cast_precision_loss)]
    Ok(dpi as f32 / 96.0)
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> crate::Result<u32> {
    let mut dpi_x = u32::default();
    let mut dpi_y = u32::default();

    unsafe {
      GetDpiForMonitor(
        HMONITOR(self.monitor_handle),
        MDT_EFFECTIVE_DPI,
        &raw mut dpi_x,
        &raw mut dpi_y,
      )
    }?;

    // Arbitrarily choose the Y DPI.
    Ok(dpi_y)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> crate::Result<bool> {
    // Check for `MONITORINFOF_PRIMARY` flag (`0x1`).
    Ok(self.monitor_info_ex()?.monitorInfo.dwFlags & 0x1 != 0)
  }

  /// Gets the display devices for this display.
  ///
  /// Enumerates monitor devices attached to the adapter associated with
  /// this display.
  pub fn devices(&self) -> crate::Result<Vec<crate::DisplayDevice>> {
    let monitor_info = self.monitor_info_ex()?;

    let adapter_name = String::from_utf16_lossy(&monitor_info.szDevice)
      .trim_end_matches('\0')
      .to_string();

    // Get the display devices associated with the display's adapter.
    let devices = (0u32..)
      .map_while(|index| {
        #[allow(clippy::cast_possible_truncation)]
        let mut device = DISPLAY_DEVICEW {
          cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
          ..Default::default()
        };

        // When passing the `EDD_GET_DEVICE_INTERFACE_NAME` flag, the
        // returned `DISPLAY_DEVICEW` will contain the device interface
        // path in the `DeviceID` field.
        unsafe {
          EnumDisplayDevicesW(
            PCWSTR(monitor_info.szDevice.as_ptr()),
            index,
            &raw mut device,
            EDD_GET_DEVICE_INTERFACE_NAME,
          )
        }
        .as_bool()
        .then_some(device)
      })
      // Filter out any devices that are not active.
      .filter(|device| device.StateFlags & DISPLAY_DEVICE_ACTIVE != 0)
      .map(|device| {
        // NOTE: This may be an empty string for virtual display devices.
        let device_interface_path =
          String::from_utf16_lossy(&device.DeviceID)
            .trim_end_matches('\0')
            .to_string();

        DisplayDevice::new(adapter_name.clone(), device_interface_path)
          .into()
      })
      .collect();

    Ok(devices)
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
      .ok_or(crate::Error::DisplayDeviceNotFound)
  }

  /// Ref: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow
  fn monitor_info_ex(&self) -> crate::Result<MONITORINFOEXW> {
    let mut monitor_info = MONITORINFOEXW {
      monitorInfo: MONITORINFO {
        #[allow(clippy::cast_possible_truncation)]
        cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
        ..Default::default()
      },
      ..Default::default()
    };

    unsafe {
      GetMonitorInfoW(
        HMONITOR(self.monitor_handle),
        std::ptr::from_mut(&mut monitor_info).cast(),
      )
    }
    .ok()?;

    Ok(monitor_info)
  }
}

impl From<Display> for crate::Display {
  fn from(display: Display) -> Self {
    crate::Display { inner: display }
  }
}

impl PartialEq for Display {
  fn eq(&self, other: &Self) -> bool {
    self.monitor_handle == other.monitor_handle
  }
}

impl Eq for Display {}

/// Windows-specific implementation of [`DisplayDevice`].
#[derive(Clone, Debug)]
pub struct DisplayDevice {
  /// Display adapter name (e.g. `\\.\DISPLAY1`).
  adapter_name: String,

  /// Device interface path (e.g.
  /// `\\?\DISPLAY#DEL40A3#5&1234abcd&0&UID256#
  /// {e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}`).
  device_interface_path: String,
}

impl DisplayDevice {
  /// Creates a new Windows display device.
  #[must_use]
  pub fn new(adapter_name: String, device_interface_path: String) -> Self {
    Self {
      adapter_name,
      device_interface_path,
    }
  }

  /// Gets the hardware ID from the device interface path.
  fn hardware_id(&self) -> Option<String> {
    self
      .device_interface_path
      .split('#')
      .nth(1)
      .map(ToString::to_string)
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    // TODO: Display adapter name might not be unique.
    DisplayDeviceId(
      self
        .hardware_id()
        .unwrap_or_else(|| self.adapter_name.clone()),
    )
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> crate::Result<f32> {
    let orientation = unsafe {
      self
        .current_device_mode()?
        .Anonymous1
        .Anonymous2
        .dmDisplayOrientation
    };

    Ok(match orientation.0 {
      1 => 90.0,
      2 => 180.0,
      3 => 270.0,
      _ => 0.0,
    })
  }

  /// Gets the output technology.
  pub fn output_technology(
    &self,
  ) -> crate::Result<Option<OutputTechnology>> {
    // TODO: Use `DisplayConfigGetDeviceInfo` to get the output technology.
    Ok(Some(OutputTechnology::Unknown))
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> crate::Result<bool> {
    // TODO: Use `DisplayConfigGetDeviceInfo` to determine whether the
    // output technology is internal.
    Ok(false)
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> crate::Result<ConnectionState> {
    // TODO: Detect disconnected state.
    Ok(ConnectionState::Active)
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> crate::Result<Option<MirroringState>> {
    // TODO: Implement mirroring detection using
    // `DisplayConfigGetDeviceInfo`.
    Ok(None)
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> crate::Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(self.current_device_mode()?.dmDisplayFrequency as f32)
  }

  /// Gets the current device mode.
  fn current_device_mode(&self) -> crate::Result<DEVMODEW> {
    #[allow(clippy::cast_possible_truncation)]
    let mut device_mode = DEVMODEW {
      dmSize: std::mem::size_of::<DEVMODEW>() as u16,
      ..Default::default()
    };

    let wide_adapter_name = self
      .adapter_name
      .encode_utf16()
      .chain(std::iter::once(0))
      .collect::<Vec<_>>();

    unsafe {
      EnumDisplaySettingsW(
        PCWSTR(wide_adapter_name.as_ptr()),
        ENUM_CURRENT_SETTINGS,
        &raw mut device_mode,
      )
    }
    .ok()?;

    Ok(device_mode)
  }
}

impl From<DisplayDevice> for crate::DisplayDevice {
  fn from(device: DisplayDevice) -> Self {
    crate::DisplayDevice { inner: device }
  }
}

/// Windows-specific implementation of [`Dispatcher::displays`].
pub fn all_displays(_: &Dispatcher) -> crate::Result<Vec<crate::Display>> {
  let mut monitor_handles: Vec<isize> = Vec::new();

  // Callback for `EnumDisplayMonitors` to collect monitor handles.
  extern "system" fn monitor_enum_proc(
    handle: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    data: LPARAM,
  ) -> BOOL {
    let handles = data.0 as *mut Vec<isize>;

    // SAFETY: `data` is a valid pointer to the `monitor_handles` vec,
    // which outlives this callback.
    unsafe { (*handles).push(handle.0) };
    true.into()
  }

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      None,
      Some(monitor_enum_proc),
      LPARAM(std::ptr::from_mut(&mut monitor_handles) as _),
    )
  }
  .ok()?;

  Ok(
    monitor_handles
      .into_iter()
      .map(|handle| Display::new(handle).into())
      .collect(),
  )
}

/// Windows-specific implementation of [`Dispatcher::display_devices`].
pub fn all_display_devices(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::DisplayDevice>> {
  all_displays(dispatcher)?
    .into_iter()
    .map(|display| display.devices())
    .collect::<crate::Result<Vec<_>>>()
    .map(|vecs| vecs.into_iter().flatten().collect())
}

/// Windows-specific implementation of [`Dispatcher::display_from_point`].
pub fn display_from_point(
  point: Point,
  _: &Dispatcher,
) -> crate::Result<crate::Display> {
  let handle = unsafe {
    MonitorFromPoint(
      POINT {
        x: point.x,
        y: point.y,
      },
      MONITOR_DEFAULTTOPRIMARY,
    )
  };

  Ok(Display::new(handle.0).into())
}

/// Windows-specific implementation of [`Dispatcher::primary_display`].
pub fn primary_display(_: &Dispatcher) -> crate::Result<crate::Display> {
  let handle = unsafe {
    MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY)
  };

  Ok(Display::new(handle.0).into())
}

/// Windows-specific implementation of [`Dispatcher::nearest_display`].
pub fn nearest_display(
  native_window: &NativeWindow,
  _: &Dispatcher,
) -> crate::Result<crate::Display> {
  let handle = unsafe {
    MonitorFromWindow(native_window.hwnd(), MONITOR_DEFAULTTONEAREST)
  };

  Ok(Display::new(handle.0).into())
}
