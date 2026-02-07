use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW,
      GetMonitorInfoW, MonitorFromWindow, DEVMODEW, DISPLAY_DEVICEW,
      DISPLAY_DEVICE_ACTIVE, ENUM_CURRENT_SETTINGS, HDC, HMONITOR,
      MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
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
  Point, Rect, Result,
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
  /// Gets the output technology.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn output_technology(&self) -> Result<Option<OutputTechnology>>;
}

impl DisplayExtWindows for crate::Display {
  fn hmonitor(&self) -> HMONITOR {
    self.inner.hmonitor()
  }
}

impl DisplayDeviceExtWindows for crate::DisplayDevice {
  fn output_technology(&self) -> Result<Option<OutputTechnology>> {
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

  /// Gets the Windows monitor handle.
  pub fn hmonitor(&self) -> HMONITOR {
    HMONITOR(self.monitor_handle)
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    Ok(
      String::from_utf16_lossy(&self.monitor_info_ex()?.szDevice)
        .trim_end_matches('\0')
        .to_string(),
    )
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> Result<Rect> {
    let rc = self.monitor_info_ex()?.monitorInfo.rcMonitor;
    Ok(Rect::from_ltrb(rc.left, rc.top, rc.right, rc.bottom))
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> Result<Rect> {
    let rc = self.monitor_info_ex()?.monitorInfo.rcWork;
    Ok(Rect::from_ltrb(rc.left, rc.top, rc.right, rc.bottom))
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> Result<f32> {
    let dpi = self.dpi()?;
    #[allow(clippy::cast_precision_loss)]
    Ok(dpi as f32 / 96.0)
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> Result<u32> {
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

    Ok(dpi_y)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    // Check for `MONITORINFOF_PRIMARY` flag (`0x1`).
    Ok(self.monitor_info_ex()?.monitorInfo.dwFlags & 0x1 != 0)
  }

  /// Gets the display devices for this display.
  ///
  /// Enumerates monitor devices attached to the adapter associated with
  /// this display.
  pub fn devices(&self) -> Result<Vec<crate::DisplayDevice>> {
    let monitor_info = self.monitor_info_ex()?;

    let adapter_name = String::from_utf16_lossy(&monitor_info.szDevice)
      .trim_end_matches('\0')
      .to_string();

    // Get the display devices associated with the monitor's adapter.
    let devices = (0u32..)
      .map_while(|index| {
        #[allow(clippy::cast_possible_truncation)]
        let mut device = DISPLAY_DEVICEW {
          cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
          ..Default::default()
        };

        // When passing the `EDD_GET_DEVICE_INTERFACE_NAME` flag, the
        // returned `DISPLAY_DEVICEW` will contain the DOS device path in
        // the `DeviceID` field.
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
        let dos_device_path = String::from_utf16_lossy(&device.DeviceID)
          .trim_end_matches('\0')
          .to_string();

        let hardware_id = dos_device_path
          .split('#')
          .nth(1)
          .map_or_else(String::new, ToString::to_string);

        DisplayDevice {
          adapter_name: adapter_name.clone(),
          hardware_id,
          monitor_state_flags: device.StateFlags,
        }
        .into()
      })
      .collect();

    Ok(devices)
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(&self) -> Result<crate::DisplayDevice> {
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
  fn monitor_info_ex(&self) -> Result<MONITORINFOEXW> {
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

/// Windows-specific display device implementation.
///
/// Represents a physical monitor device attached to a display adapter.
#[derive(Clone, Debug)]
pub struct DisplayDevice {
  /// The adapter name (e.g., `\\.\DISPLAY1`).
  pub(crate) adapter_name: String,

  /// The monitor hardware ID (e.g., `DEL4042`).
  pub(crate) hardware_id: String,

  /// Monitor-level state flags from `EnumDisplayDevicesW`.
  pub(crate) monitor_state_flags: u32,
}

impl DisplayDevice {
  /// Creates a new Windows display device.
  #[must_use]
  pub fn new(
    adapter_name: String,
    hardware_id: String,
    monitor_state_flags: u32,
  ) -> Self {
    Self {
      adapter_name,
      hardware_id,
      monitor_state_flags,
    }
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    DisplayDeviceId(self.hardware_id.clone())
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    let orientation = self.current_device_mode()?.dmDisplayOrientation;

    Ok(match orientation {
      0 => 0.0,
      1 => 90.0,
      2 => 180.0,
      3 => 270.0,
      _ => 0.0,
    })
  }

  /// Gets the output technology.
  pub fn output_technology(&self) -> Result<Option<OutputTechnology>> {
    // TODO: Use `DisplayConfigGetDeviceInfo` to get the
    // actual output technology.
    Ok(Some(OutputTechnology::Unknown))
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> Result<bool> {
    // TODO: Use `DisplayConfigGetDeviceInfo` to determine
    // whether the output technology is internal.
    Ok(false)
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
    // TODO: Detect disconnected state.
    if self.monitor_state_flags & DISPLAY_DEVICE_ACTIVE != 0 {
      Ok(ConnectionState::Active)
    } else {
      Ok(ConnectionState::Inactive)
    }
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<Option<MirroringState>> {
    // TODO: Implement proper mirroring detection using
    // `DisplayConfigGetDeviceInfo`.
    Ok(None)
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    #[allow(clippy::cast_possible_truncation)]
    Ok(self.current_device_mode()?.dmDisplayFrequency as f32)
  }

  /// Gets the current device mode.
  fn current_device_mode(&self) -> Result<DEVMODEW> {
    #[allow(clippy::cast_possible_truncation)]
    let mut device_mode = DEVMODEW {
      dmSize: std::mem::size_of::<DEVMODEW>() as u32,
      ..Default::default()
    };

    let wide_name: Vec<u16> = self
      .adapter_name
      .encode_utf16()
      .chain(std::iter::once(0))
      .collect();

    unsafe {
      EnumDisplaySettingsW(
        PCWSTR(wide_name.as_ptr()),
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
pub fn all_displays(
  _: &crate::Dispatcher,
) -> crate::Result<Vec<crate::Display>> {
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
///
/// Enumerates all active monitor devices across all display adapters.
pub fn all_display_devices(
  _: &crate::Dispatcher,
) -> crate::Result<Vec<crate::DisplayDevice>> {
  let mut devices = Vec::new();

  // Enumerate display adapters (level 1).
  for adapter_index in 0u32.. {
    #[allow(clippy::cast_possible_truncation)]
    let mut adapter = DISPLAY_DEVICEW {
      cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
      ..Default::default()
    };

    let found = unsafe {
      EnumDisplayDevicesW(
        PCWSTR::null(),
        adapter_index,
        &raw mut adapter,
        0,
      )
    };

    if !found.as_bool() {
      break;
    }

    // Skip inactive adapters.
    if adapter.StateFlags & DISPLAY_DEVICE_ACTIVE == 0 {
      continue;
    }

    let adapter_name = String::from_utf16_lossy(&adapter.DeviceName)
      .trim_end_matches('\0')
      .to_string();

    // Enumerate child monitor devices (level 2).
    for monitor_index in 0u32.. {
      #[allow(clippy::cast_possible_truncation)]
      let mut monitor = DISPLAY_DEVICEW {
        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
      };

      // Due to `EDD_GET_DEVICE_INTERFACE_NAME`, the
      // `DeviceID` field contains the DOS device path.
      let found = unsafe {
        EnumDisplayDevicesW(
          PCWSTR(adapter.DeviceName.as_ptr()),
          monitor_index,
          &raw mut monitor,
          EDD_GET_DEVICE_INTERFACE_NAME,
        )
      };

      if !found.as_bool() {
        break;
      }

      // Skip inactive monitors.
      if monitor.StateFlags & DISPLAY_DEVICE_ACTIVE == 0 {
        continue;
      }

      let device_path = String::from_utf16_lossy(&monitor.DeviceID)
        .trim_end_matches('\0')
        .to_string();

      let hardware_id = device_path
        .split('#')
        .nth(1)
        .map_or_else(String::new, ToString::to_string);

      devices.push(
        DisplayDevice::new(
          adapter_name.clone(),
          hardware_id,
          monitor.StateFlags,
        )
        .into(),
      );
    }
  }

  Ok(devices)
}

/// Windows-specific implementation of [`Dispatcher::display_from_point`].
pub fn display_from_point(
  point: Point,
  _: &crate::Dispatcher,
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
pub fn primary_display(
  _: &crate::Dispatcher,
) -> crate::Result<crate::Display> {
  let handle = unsafe {
    MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY)
  };

  Ok(Display::new(handle.0).into())
}

/// Windows-specific implementation of [`Dispatcher::nearest_display`].
pub fn nearest_display(
  native_window: &crate::NativeWindow,
  _: &crate::Dispatcher,
) -> crate::Result<crate::Display> {
  let handle = unsafe {
    MonitorFromWindow(native_window.hwnd(), MONITOR_DEFAULTTONEAREST)
  };

  Ok(Display::new(handle.0).into())
}
