use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW,
      GetMonitorInfoW, MonitorFromWindow, DEVMODEW, DISPLAY_DEVICEW,
      DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_MIRRORING_DRIVER, HDC,
      HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};
use wm_common::{Point, Rect};

use crate::{
  display::{
    DisplayConnection, DisplayDeviceData, DisplayDeviceId,
    DisplayDeviceState, DisplayId, MirroringState, PhysicalDeviceData,
    VirtualDeviceData,
  },
  platform_ext::windows::{DisplayOrientation, WindowsDisplaySettings},
  Result,
};

pub type DisplayId = isize;
pub type DisplayDeviceId = isize;

/// Windows-specific display implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
  pub(crate) monitor_handle: isize,
  pub(crate) device_name: String,
  pub(crate) device_id: String,
}

impl Display {
  /// Creates a new Windows display from monitor handle and device info.
  #[must_use]
  pub fn new(
    monitor_handle: isize,
    device_name: String,
    device_id: String,
  ) -> Self {
    Self {
      monitor_handle,
      device_name,
      device_id,
    }
  }

  /// Gets the unique identifier for this display.
  pub fn id(&self) -> DisplayId {
    DisplayId::new(format!("windows:{}", self.monitor_handle))
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

  /// Gets the display resolution in pixels.
  pub fn resolution(&self) -> Result<(u32, u32)> {
    let bounds = self.bounds()?;
    Ok((bounds.width().try_into()?, bounds.height().try_into()?))
  }

  /// Gets the current refresh rate in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    let device_mode = self.get_current_display_mode()?;
    Ok(device_mode.dmDisplayFrequency as f32)
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

  /// Gets the bit depth of the display.
  pub fn bit_depth(&self) -> Result<u32> {
    let device_mode = self.get_current_display_mode()?;
    Ok(device_mode.dmBitsPerPel)
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    let monitor_info = self.get_monitor_info()?;
    Ok(monitor_info.monitorInfo.dwFlags & 0x1 != 0) // MONITORINFOF_PRIMARY
  }

  /// Returns whether this display supports HDR.
  pub fn is_hdr_capable(&self) -> Result<bool> {
    // Would need to check display capabilities via DXGI or similar
    Ok(false)
  }

  /// Gets all supported refresh rates for this display.
  pub fn supported_refresh_rates(&self) -> Result<Vec<f32>> {
    let mut refresh_rates = Vec::new();
    let mut mode_index = 0u32;

    loop {
      let mut device_mode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>().try_into()?,
        ..Default::default()
      };

      let result = unsafe {
        EnumDisplaySettingsW(
          PCWSTR(
            self.device_name.encode_utf16().collect::<Vec<_>>().as_ptr(),
          ),
          mode_index,
          &raw mut device_mode,
        )
      };

      if !result.as_bool() {
        break;
      }

      refresh_rates.push(device_mode.dmDisplayFrequency as f32);
      mode_index += 1;
    }

    refresh_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    refresh_rates.dedup();
    Ok(refresh_rates)
  }

  /// Gets all supported resolutions for this display.
  pub fn supported_resolutions(&self) -> Result<Vec<(u32, u32)>> {
    let device_name = self.get_device_name()?;
    let mut resolutions = Vec::new();
    let mut mode_index = 0u32;

    loop {
      let mut device_mode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>().try_into()?,
        ..Default::default()
      };

      let result = unsafe {
        EnumDisplaySettingsW(
          PCWSTR(device_name.encode_utf16().collect::<Vec<_>>().as_ptr()),
          mode_index,
          &raw mut device_mode,
        )
      };

      if !result.as_bool() {
        break;
      }

      resolutions
        .push((device_mode.dmPelsWidth, device_mode.dmPelsHeight));
      mode_index += 1;
    }

    resolutions.sort();
    resolutions.dedup();
    Ok(resolutions)
  }

  /// Gets the ID of the device driving this display.
  pub fn device_id(&self) -> DisplayDeviceId {
    self
      .get_device_name()
      .map(|name| DisplayDeviceId::new(name))
      .unwrap_or_else(|_| {
        DisplayDeviceId::new(format!("unknown:{}", self.monitor_handle))
      })
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
      if device.mirroring_state()? == MirroringState::None
        || device.mirroring_state()? == MirroringState::Source
      {
        return Ok(Some(device));
      }
    }

    Ok(None)
  }

  /// Gets the Windows monitor handle.
  pub fn hmonitor(&self) -> HMONITOR {
    HMONITOR(self.monitor_handle)
  }

  /// Gets Windows-specific display settings.
  pub fn display_settings(&self) -> Result<WindowsDisplaySettings> {
    let device_mode = self.get_current_display_mode()?;

    let orientation = match device_mode.dmDisplayOrientation {
      0 => DisplayOrientation::Default,
      1 => DisplayOrientation::Rotate90,
      2 => DisplayOrientation::Rotate180,
      3 => DisplayOrientation::Rotate270,
      _ => DisplayOrientation::Default,
    };

    Ok(WindowsDisplaySettings {
      width: device_mode.dmPelsWidth,
      height: device_mode.dmPelsHeight,
      bits_per_pixel: device_mode.dmBitsPerPel,
      refresh_rate: device_mode.dmDisplayFrequency,
      orientation,
    })
  }

  /// Gets the monitor info structure from Windows API.
  fn get_monitor_info(&self) -> Result<MONITORINFOEXW> {
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

  /// Gets the current display mode.
  fn get_current_display_mode(&self) -> Result<DEVMODEW> {
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

/// Windows-specific display device implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    DisplayDeviceId::new(&self.hardware_id)
  }

  /// Gets the device name.
  pub fn name(&self) -> Result<String> {
    self.get_device_string()
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    let device_mode = self.get_current_device_mode()?;
    let orientation = device_mode.dmDisplayOrientation;

    Ok(match orientation {
      0 => 0.0,   // Default
      1 => 90.0,  // 90 degrees
      2 => 180.0, // 180 degrees
      3 => 270.0, // 270 degrees
      _ => 0.0,
    })
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
    let state_flags = self.get_state_flags()?;
    if state_flags & DISPLAY_DEVICE_ACTIVE != 0 {
      Ok(ConnectionState::Active)
    } else {
      Ok(ConnectionState::Inactive)
    }
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    let device_mode = self.get_current_device_mode()?;
    Ok(device_mode.dmDisplayFrequency as f32)
  }

  /// Returns whether this is a built-in device.
  pub fn is_builtin(&self) -> Result<bool> {
    let device_string = self.get_device_string()?;
    let device_lower = device_string.to_lowercase();
    Ok(
      device_lower.contains("laptop")
        || device_lower.contains("internal")
        || device_lower.contains("built-in"),
    )
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<MirroringState> {
    let state_flags = self.get_state_flags()?;

    // TODO
    if state_flags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0 {
      Ok(MirroringState::Target)
    } else {
      Ok(MirroringState::None)
    }
  }

  /// Gets the device-specific data.
  pub fn data(&self) -> Result<DisplayDeviceData> {
    let device_string = self.get_device_string()?;
    let adapter_name = self.get_adapter_name()?;

    // Determine if this is a virtual or physical device
    let is_virtual = device_string.contains("Virtual")
      || device_string.contains("Software")
      || adapter_name.contains("Remote");

    if is_virtual {
      Ok(DisplayDeviceData::Virtual(VirtualDeviceData {
        driver_name: Some(adapter_name),
        virtual_adapter_id: Some(self.hardware_id.clone()),
      }))
    } else {
      let connection_type = self.determine_connection_type(&device_string);
      let is_builtin = self.is_builtin()?;
      let output_technology = self.output_technology()?;

      Ok(DisplayDeviceData::Physical(PhysicalDeviceData {
        vendor: self.extract_vendor(&device_string),
        model: Some(device_string),
        serial_number: None, // Would need registry lookup
        hardware_id: Some(self.hardware_id.clone()),
        edid_data: None,        // Would need registry lookup
        physical_size_mm: None, // Would need EDID parsing
        connection_type: Some(connection_type),
        output_technology,
        is_builtin,
      }))
    }
  }

  /// Gets the output technology (Windows-specific).
  pub fn output_technology(&self) -> Result<Option<String>> {
    // This would require querying Windows APIs for output technology
    // For now, return None as a placeholder
    Ok(None)
  }

  // Windows-specific methods

  /// Gets the device path for the display adapter.
  pub fn adapter_device_path(&self) -> Result<Option<String>> {
    Ok(Some(format!("\\\\?\\{}", self.device_name)))
  }

  /// Gets the device instance ID.
  pub fn device_instance_id(&self) -> Result<Option<String>> {
    Ok(Some(self.device_id.clone()))
  }

  /// Gets the hardware vendor and device IDs.
  pub fn hardware_ids(&self) -> Result<Option<(u16, u16)>> {
    // Parse hardware ID from device_id string
    // Format is typically "PCI\VEN_####&DEV_####..."
    if let Some(ven_pos) = self.device_id.find("VEN_") {
      if let Some(dev_pos) = self.device_id.find("&DEV_") {
        let vendor_str = &self.device_id[ven_pos + 4..ven_pos + 8];
        let device_str = &self.device_id[dev_pos + 5..dev_pos + 9];

        if let (Ok(vendor_id), Ok(device_id)) = (
          u16::from_str_radix(vendor_str, 16),
          u16::from_str_radix(device_str, 16),
        ) {
          return Ok(Some((vendor_id, device_id)));
        }
      }
    }
    Ok(None)
  }

  /// Gets the registry key for device properties.
  pub fn registry_key(&self) -> Result<Option<String>> {
    Ok(Some(self.device_key.clone()))
  }

  /// Checks if the device is built-in using Windows output type.
  pub fn is_builtin_windows(&self) -> Result<bool> {
    Ok(self.is_builtin_device())
  }

  /// Gets the device string from Windows API.
  fn get_device_string(&self) -> Result<String> {
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

  /// Gets the adapter name from Windows API.
  fn get_adapter_name(&self) -> Result<String> {
    // This would require additional Windows API calls
    // For now, return a placeholder
    Ok("Unknown Adapter".to_string())
  }

  /// Gets the state flags from Windows API.
  fn get_state_flags(&self) -> Result<u32> {
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
  fn get_current_device_mode(&self) -> Result<DEVMODEW> {
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

  /// Determines the connection type based on device information.
  fn determine_connection_type(
    &self,
    device_string: &str,
  ) -> DisplayConnection {
    let device_lower = device_string.to_lowercase();

    if device_lower.contains("laptop") || device_lower.contains("internal")
    {
      DisplayConnection::Internal
    } else if device_lower.contains("hdmi") {
      DisplayConnection::HDMI
    } else if device_lower.contains("displayport")
      || device_lower.contains("dp")
    {
      DisplayConnection::DisplayPort
    } else if device_lower.contains("dvi") {
      DisplayConnection::DVI
    } else if device_lower.contains("vga") {
      DisplayConnection::VGA
    } else if device_lower.contains("usb") {
      DisplayConnection::USB
    } else if device_lower.contains("thunderbolt") {
      DisplayConnection::Thunderbolt
    } else {
      DisplayConnection::Unknown
    }
  }

  /// Extracts vendor name from device information.
  fn extract_vendor(&self, device_string: &str) -> Option<String> {
    // Common vendor patterns in device strings
    let device_lower = device_string.to_lowercase();

    if device_lower.contains("nvidia") {
      Some("NVIDIA".to_string())
    } else if device_lower.contains("amd")
      || device_lower.contains("radeon")
    {
      Some("AMD".to_string())
    } else if device_lower.contains("intel") {
      Some("Intel".to_string())
    } else {
      None
    }
  }
}

/// Gets all active displays on Windows.
pub fn all_displays() -> Result<Vec<Display>> {
  let monitor_handles = get_all_monitor_handles()?;
  let mut displays = Vec::new();

  for handle in monitor_handles {
    displays.push(Display::new(handle));
  }

  Ok(displays)
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
    let device_string =
      String::from_utf16_lossy(&display_device.DeviceString)
        .trim_end_matches('\0')
        .to_string();
    let device_id = String::from_utf16_lossy(&display_device.DeviceID)
      .trim_end_matches('\0')
      .to_string();
    let device_key = String::from_utf16_lossy(&display_device.DeviceKey)
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

  // Fall back to primary display
  primary_display()
}

/// Gets primary display on Windows.
pub fn primary_display() -> Result<Display> {
  let displays = all_displays()?;

  for display in displays {
    if display.is_primary()? {
      return Ok(display);
    }
  }

  // If no primary found, return first display
  all_displays()?.into_iter().next().ok_or_else(|| {
    crate::Error::Anyhow(anyhow::anyhow!("No displays found"))
  })
}

/// Gets all available monitor handles.
fn get_all_monitor_handles() -> Result<Vec<isize>> {
  let mut monitors: Vec<isize> = Vec::new();

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      None,
      Some(monitor_enum_proc),
      LPARAM(std::ptr::from_mut(&mut monitors) as _),
    )
  }
  .ok()?;

  Ok(monitors)
}

/// Callback for `EnumDisplayMonitors` to collect monitor handles.
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

/// Gets monitor info for a specific handle.
fn get_monitor_info_for_handle(handle: isize) -> Result<MONITORINFOEXW> {
  let mut monitor_info = MONITORINFOEXW {
    monitorInfo: MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFOEXW>().try_into()?,
      ..Default::default()
    },
    ..Default::default()
  };

  unsafe {
    GetMonitorInfoW(
      HMONITOR(handle),
      std::ptr::from_mut(&mut monitor_info).cast(),
    )
  }
  .ok()?;

  Ok(monitor_info)
}
