use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW,
      MonitorFromWindow, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
      DISPLAY_DEVICE_MIRRORING_DRIVER, HDC, HMONITOR, MONITORINFO,
      MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};
use wm_common::{Point, Rect};

use crate::{MonitorState, Result};

/// Windows-specific monitor implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMonitor {
  pub handle: isize,
}

impl NativeMonitor {
  /// Creates a new `NativeMonitor` for the given handle.
  #[must_use]
  pub fn new(handle: isize) -> Self {
    Self { handle }
  }

  /// Gets the device name of the monitor.
  pub fn device_name(&self) -> Result<String> {
    let monitor_info = self.get_monitor_info()?;
    Ok(String::from_utf16_lossy(&monitor_info.szDevice))
  }

  /// Gets the hardware identifier for the monitor.
  pub fn hardware_id(&self) -> Result<Option<String>> {
    let monitor_info = self.get_monitor_info()?;
    let display_devices = self.get_display_devices(&monitor_info)?;

    // Get the hardware ID from the first valid device
    let hardware_id = display_devices.first().and_then(|device| {
      let device_path = String::from_utf16_lossy(&device.DeviceID)
        .trim_end_matches('\0')
        .to_string();

      device_path
        .split('#')
        .nth(1)
        .map(std::string::ToString::to_string)
    });

    Ok(hardware_id)
  }

  /// Gets the full bounds rectangle of the monitor.
  pub fn rect(&self) -> Result<Rect> {
    let monitor_info = self.get_monitor_info()?;
    let rc_monitor = monitor_info.monitorInfo.rcMonitor;
    Ok(Rect::from_ltrb(
      rc_monitor.left,
      rc_monitor.top,
      rc_monitor.right,
      rc_monitor.bottom,
    ))
  }

  /// Gets the working area rectangle (excluding taskbar and docked
  /// windows).
  pub fn working_rect(&self) -> Result<Rect> {
    let monitor_info = self.get_monitor_info()?;
    let rc_work = monitor_info.monitorInfo.rcWork;
    Ok(Rect::from_ltrb(
      rc_work.left,
      rc_work.top,
      rc_work.right,
      rc_work.bottom,
    ))
  }

  /// Gets the DPI for the monitor.
  pub fn dpi(&self) -> Result<u32> {
    let mut dpi_x = u32::default();
    let mut dpi_y = u32::default();

    unsafe {
      GetDpiForMonitor(
        HMONITOR(self.handle),
        MDT_EFFECTIVE_DPI,
        &raw mut dpi_x,
        &raw mut dpi_y,
      )
    }?;

    // Return the Y DPI (could also use X DPI)
    Ok(dpi_y)
  }

  /// Gets the scale factor for the monitor.
  pub fn scale_factor(&self) -> Result<f32> {
    let dpi = self.dpi()?;
    #[allow(clippy::cast_precision_loss)]
    Ok(dpi as f32 / 96.0)
  }

  /// Gets the current state of the monitor.
  pub fn state(&self) -> Result<MonitorState> {
    let monitor_info = self.get_monitor_info()?;
    let display_devices = self.get_display_devices(&monitor_info)?;

    // Check the state of the display devices
    for device in display_devices {
      if device.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0 {
        return Ok(MonitorState::Mirroring);
      }
      if device.StateFlags & DISPLAY_DEVICE_ACTIVE != 0 {
        return Ok(MonitorState::Active);
      }
    }

    Ok(MonitorState::Inactive)
  }

  /// Returns whether this is the primary monitor.
  pub fn is_primary(&self) -> Result<bool> {
    let monitor_info = self.get_monitor_info()?;
    Ok(monitor_info.monitorInfo.dwFlags & 0x1 != 0) // MONITORINFOF_PRIMARY
  }

  /// Gets the monitor info structure from Windows API.
  fn get_monitor_info(&self) -> Result<MONITORINFOEXW> {
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
        HMONITOR(self.handle),
        std::ptr::from_mut(&mut monitor_info).cast(),
      )
    }
    .ok()?;

    Ok(monitor_info)
  }

  /// Gets the display devices associated with this monitor.
  fn get_display_devices(
    &self,
    monitor_info: &MONITORINFOEXW,
  ) -> Result<Vec<DISPLAY_DEVICEW>> {
    let display_devices = (0..)
      .map_while(|index| {
        #[allow(clippy::cast_possible_truncation)]
        let mut display_device = DISPLAY_DEVICEW {
          cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
          ..Default::default()
        };

        unsafe {
          EnumDisplayDevicesW(
            PCWSTR(monitor_info.szDevice.as_ptr()),
            index,
            &raw mut display_device,
            EDD_GET_DEVICE_INTERFACE_NAME,
          )
        }
        .as_bool()
        .then_some(display_device)
      })
      .collect();

    Ok(display_devices)
  }
}

/// Gets all monitors, including active, mirroring, and inactive ones.
pub fn all_monitors() -> Result<Vec<NativeMonitor>> {
  Ok(
    get_all_monitor_handles()?
      .into_iter()
      .map(NativeMonitor::new)
      .collect(),
  )
}

/// Gets the monitor containing the specified point.
pub fn monitor_from_point(point: Point) -> Result<NativeMonitor> {
  let monitors = all_monitors()?;

  for monitor in &monitors {
    let rect = monitor.rect()?;
    if rect.contains_point(&point) {
      return Ok(monitor.clone());
    }
  }

  // Fall back to primary monitor
  primary_monitor()
}

/// Gets the primary monitor.
pub fn primary_monitor() -> Result<NativeMonitor> {
  let monitors = all_monitors()?;

  for monitor in monitors {
    if monitor.is_primary()? {
      return Ok(monitor);
    }
  }

  // If no primary found, use the first monitor
  all_monitors()?
    .into_iter()
    .next()
    .ok_or_else(|| anyhow::anyhow!("No monitors found").into())
}

/// Gets the monitor nearest to the specified window.
pub fn nearest_monitor(window_handle: isize) -> Result<NativeMonitor> {
  let handle = unsafe {
    MonitorFromWindow(HWND(window_handle), MONITOR_DEFAULTTONEAREST)
  };

  Ok(NativeMonitor::new(handle.0))
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_monitor_creation() {
    let monitor = NativeMonitor::new(1);
    assert_eq!(monitor.handle, 1);

    let monitor2 = NativeMonitor::new(1);
    let monitor3 = NativeMonitor::new(2);
    assert_eq!(monitor, monitor2);
    assert_ne!(monitor, monitor3);
  }
}
