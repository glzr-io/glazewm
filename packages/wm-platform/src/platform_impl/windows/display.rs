use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW,
      GetMonitorInfoW, DEVMODEW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
      DISPLAY_DEVICE_MIRRORING_DRIVER, HDC, HMONITOR, MONITORINFO,
      MONITORINFOEXW,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};
use wm_common::{Point, Rect};

use crate::{
  display::{ConnectionState, DisplayDeviceId, DisplayId, MirroringState},
  error::{PlatformError, Result},
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
  pub fn output_technology(&self) -> Result<Option<String>> {
    self.inner.output_technology()
  }
}

impl DisplayExtWindows for Display {
  fn hmonitor(&self) -> HMONITOR {
    self.inner.hmonitor()
  }

  fn display_settings(&self) -> Result<WindowsDisplaySettings> {
    self.inner.display_settings()
  }
}

impl DisplayDeviceExtWindows for DisplayDevice {
  fn is_builtin(&self) -> Result<bool> {
    self.inner.is_builtin()
  }
}

/// Windows-specific display implementation.
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
  pub fn hmonitor(&self) -> isize {
    self.monitor_handle
  }

  /// Gets the NSScreen instance (not available on Windows).
  #[cfg(target_os = "macos")]
  pub fn ns_screen(&self) -> &crate::platform_impl::NSScreenRef {
    unreachable!("NSScreen not available on Windows")
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    let monitor_info = self.get_monitor_info()?;
    let device_name = String::from_utf16_lossy(&monitor_info.szDevice)
      .trim_end_matches('\0')
      .to_string();
    Ok(device_name)
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> Result<Rect> {
    let monitor_info = self.get_monitor_info()?;
    let rc_monitor = monitor_info.monitorInfo.rcMonitor;
    Ok(Rect::from_ltrb(
      rc_monitor.left,
      rc_monitor.top,
      rc_monitor.right,
      rc_monitor.bottom,
    ))
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> Result<Rect> {
    let monitor_info = self.get_monitor_info()?;
    let rc_work = monitor_info.monitorInfo.rcWork;
    Ok(Rect::from_ltrb(
      rc_work.left,
      rc_work.top,
      rc_work.right,
      rc_work.bottom,
    ))
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
    let monitor_info = self.get_monitor_info()?;
    Ok(monitor_info.monitorInfo.dwFlags & 0x1 != 0) // MONITORINFOF_PRIMARY
  }

  /// Gets the display devices for this display.
  pub fn devices(&self) -> Result<Vec<crate::display::DisplayDevice>> {
    let device_name = self.get_device_name()?;
    let all_devices = all_display_devices()?;

    // Filter devices that match this display's device name
    Ok(
      all_devices
        .into_iter()
        .filter(|device| device.device_name == device_name)
        .map(crate::display::DisplayDevice::from_platform_impl)
        .collect(),
    )
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(
    &self,
  ) -> Result<Option<crate::display::DisplayDevice>> {
    let devices = self.devices()?;

    // Find first device that is not mirroring
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

  /// Gets the monitor info structure from Windows API.
  fn monitor_info(&self) -> Result<MONITORINFOEXW> {
    let mut monitor_info = MONITORINFOEXW {
      monitorInfo: MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFOEXW>().try_into()?,
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

  /// Gets the device name for this display.
  fn device_name(&self) -> Result<String> {
    let monitor_info = self.get_monitor_info()?;
    Ok(
      String::from_utf16_lossy(&monitor_info.szDevice)
        .trim_end_matches('\0')
        .to_string(),
    )
  }
}

/// Windows-specific display device implementation.
#[derive(Clone, Debug)]
pub struct DisplayDevice {
  pub(crate) device_name: String,
  pub(crate) hardware_id: String,
}

impl DisplayDevice {
  /// Creates a new Windows display device.
  #[must_use]
  pub fn new(device_name: String, hardware_id: String) -> Self {
    Self {
      device_name,
      hardware_id,
    }
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    DisplayDeviceId(self.hardware_id.clone())
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    let device_mode = self.current_device_mode()?;
    let orientation = device_mode.dmDisplayOrientation;

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
    todo!()
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> Result<bool> {
    todo!()
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
    let state_flags = self.get_state_flags()?;
    // TODO: Get whether disconnected.
    if state_flags & DISPLAY_DEVICE_ACTIVE != 0 {
      Ok(ConnectionState::Active)
    } else {
      Ok(ConnectionState::Inactive)
    }
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<Option<MirroringState>> {
    let state_flags = self.get_state_flags()?;

    // TODO: Implement this properly.
    if state_flags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0 {
      Ok(Some(MirroringState::Target))
    } else {
      Ok(None)
    }
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    let device_mode = self.current_device_mode()?;
    Ok(device_mode.dmDisplayFrequency as f32)
  }

  /// Gets the device string from Windows API.
  fn device_string(&self) -> Result<String> {
    let mut display_device = DISPLAY_DEVICEW {
      cb: std::mem::size_of::<DISPLAY_DEVICEW>().try_into()?,
      ..Default::default()
    };

    // Find the device by device name
    let mut device_index = 0u32;
    loop {
      let result = unsafe {
        EnumDisplayDevicesW(
          PCWSTR::null(),
          device_index,
          &raw mut display_device,
          0,
        )
      };

      if !result.as_bool() {
        break;
      }

      let current_device_name =
        String::from_utf16_lossy(&display_device.DeviceName)
          .trim_end_matches('\0')
          .to_string();

      if current_device_name == self.device_name {
        return Ok(
          String::from_utf16_lossy(&display_device.DeviceString)
            .trim_end_matches('\0')
            .to_string(),
        );
      }

      device_index += 1;
    }

    Ok("Unknown Device".to_string())
  }

  /// Gets the state flags from Windows API.
  fn state_flags(&self) -> Result<u32> {
    let mut display_device = DISPLAY_DEVICEW {
      cb: std::mem::size_of::<DISPLAY_DEVICEW>().try_into()?,
      ..Default::default()
    };

    // Find the device by device name
    let mut device_index = 0u32;
    loop {
      let result = unsafe {
        EnumDisplayDevicesW(
          PCWSTR::null(),
          device_index,
          &raw mut display_device,
          0,
        )
      };

      if !result.as_bool() {
        break;
      }

      let current_device_name =
        String::from_utf16_lossy(&display_device.DeviceName)
          .trim_end_matches('\0')
          .to_string();

      if current_device_name == self.device_name {
        return Ok(display_device.StateFlags);
      }

      device_index += 1;
    }

    Ok(0)
  }

  /// Gets the current device mode from Windows API.
  fn current_device_mode(&self) -> Result<DEVMODEW> {
    let mut device_mode = DEVMODEW {
      dmSize: std::mem::size_of::<DEVMODEW>().try_into()?,
      ..Default::default()
    };

    unsafe {
      EnumDisplaySettingsW(
        PCWSTR(
          self.device_name.encode_utf16().collect::<Vec<_>>().as_ptr(),
        ),
        u32::MAX, // ENUM_CURRENT_SETTINGS
        &raw mut device_mode,
      )
    }
    .ok()?;

    Ok(device_mode)
  }
}

/// Gets all active displays on Windows.
pub fn all_displays() -> Result<Vec<Display>> {
  let mut monitor_handles: Vec<isize> = Vec::new();

  // Callback for `EnumDisplayMonitors` to collect monitor handles.
  extern "system" fn monitor_enum_proc(
    handle: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    data: LPARAM,
  ) -> BOOL {
    let handles = data.0 as *mut Vec<isize>;
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

  Ok(monitor_handles.into_iter().map(Display::new).collect())
}

/// Gets all display devices on Windows.
pub fn all_display_devices() -> Result<Vec<DisplayDevice>> {
  let mut devices = Vec::new();
  let mut device_index = 0u32;

  loop {
    let mut display_device = DISPLAY_DEVICEW {
      cb: std::mem::size_of::<DISPLAY_DEVICEW>().try_into()?,
      ..Default::default()
    };

    let result = unsafe {
      EnumDisplayDevicesW(
        PCWSTR::null(),
        device_index,
        &raw mut display_device,
        EDD_GET_DEVICE_INTERFACE_NAME,
      )
    };

    if !result.as_bool() {
      break;
    }

    let device_name = String::from_utf16_lossy(&display_device.DeviceName)
      .trim_end_matches('\0')
      .to_string();
    let device_id = String::from_utf16_lossy(&display_device.DeviceID)
      .trim_end_matches('\0')
      .to_string();

    devices.push(DisplayDevice::new(device_name, device_id));

    device_index += 1;
  }

  Ok(devices)
}

/// Gets display from point on Windows.
pub fn display_from_point(point: Point) -> Result<Display> {
  let displays = all_displays()?;

  for display in &displays {
    let bounds = display.bounds()?;
    if bounds.contains_point(&point) {
      return Ok(display.clone());
    }
  }

  Err(crate::Error::DisplayNotFound.into())
}

/// Gets primary display on Windows.
pub fn primary_display() -> Result<Display> {
  let displays = all_displays()?;

  for display in displays {
    if display.is_primary()? {
      return Ok(display);
    }
  }

  Err(crate::Error::PrimaryDisplayNotFound.into())
}

/// Gets all available monitor handles.
fn all_monitor_handles() -> Result<Vec<isize>> {}
