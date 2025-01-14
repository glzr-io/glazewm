use std::cell::OnceCell;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW,
      MonitorFromWindow, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, HDC,
      HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};
use wm_common::Rect;

#[derive(Clone, Debug)]
pub struct NativeMonitor {
  pub handle: isize,
  info: OnceCell<MonitorInfo>,
}

#[derive(Clone, Debug)]
struct MonitorInfo {
  device_name: String,
  device_path: Option<String>,
  hardware_id: Option<String>,
  rect: Rect,
  working_rect: Rect,
  dpi: u32,
  scale_factor: f32,
}

impl NativeMonitor {
  #[must_use]
  pub fn new(handle: isize) -> Self {
    Self {
      handle,
      info: OnceCell::new(),
    }
  }

  pub fn device_name(&self) -> anyhow::Result<&String> {
    self.monitor_info().map(|info| &info.device_name)
  }

  pub fn device_path(&self) -> anyhow::Result<Option<&String>> {
    self.monitor_info().map(|info| info.device_path.as_ref())
  }

  pub fn hardware_id(&self) -> anyhow::Result<Option<&String>> {
    self.monitor_info().map(|info| info.hardware_id.as_ref())
  }

  pub fn rect(&self) -> anyhow::Result<&Rect> {
    self.monitor_info().map(|info| &info.rect)
  }

  pub fn working_rect(&self) -> anyhow::Result<&Rect> {
    self.monitor_info().map(|info| &info.working_rect)
  }

  pub fn dpi(&self) -> anyhow::Result<u32> {
    self.monitor_info().map(|info| info.dpi)
  }

  pub fn scale_factor(&self) -> anyhow::Result<f32> {
    self.monitor_info().map(|info| info.scale_factor)
  }

  fn monitor_info(&self) -> anyhow::Result<&MonitorInfo> {
    self.info.get_or_try_init(|| {
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

      // Get the display devices associated with the monitor.
      let mut display_devices = (0..)
        .map_while(|index| {
          #[allow(clippy::cast_possible_truncation)]
          let mut display_device = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
          };

          // Due to the `EDD_GET_DEVICE_INTERFACE_NAME` flag, the device
          // struct will contain the DOS device path under the `DeviceId`
          // field.
          unsafe {
            EnumDisplayDevicesW(
              PCWSTR(monitor_info.szDevice.as_ptr()),
              index,
              &mut display_device,
              EDD_GET_DEVICE_INTERFACE_NAME,
            )
          }
          .as_bool()
          .then_some(display_device)
        })
        // Filter out any devices that are not active.
        .filter(|device| device.StateFlags & DISPLAY_DEVICE_ACTIVE != 0);

      // Get the device path and hardware ID from the first valid device.
      let (device_path, hardware_id) =
        display_devices.next().map_or((None, None), |device| {
          let device_path = String::from_utf16_lossy(&device.DeviceID)
            .trim_end_matches('\0')
            .to_string();

          let hardware_id = device_path
            .split('#')
            .collect::<Vec<_>>()
            .get(1)
            .map(|id| (*id).to_string());

          (Some(device_path), hardware_id)
        });

      let device_name = String::from_utf16_lossy(&monitor_info.szDevice);
      let dpi = monitor_dpi(self.handle)?;
      #[allow(clippy::cast_precision_loss)]
      let scale_factor = dpi as f32 / 96.0;

      let rc_monitor = monitor_info.monitorInfo.rcMonitor;
      let rect = Rect::from_ltrb(
        rc_monitor.left,
        rc_monitor.top,
        rc_monitor.right,
        rc_monitor.bottom,
      );

      let rc_work = monitor_info.monitorInfo.rcWork;
      let working_rect = Rect::from_ltrb(
        rc_work.left,
        rc_work.top,
        rc_work.right,
        rc_work.bottom,
      );

      Ok(MonitorInfo {
        device_name,
        device_path,
        hardware_id,
        rect,
        working_rect,
        dpi,
        scale_factor,
      })
    })
  }
}

impl PartialEq for NativeMonitor {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeMonitor {}

/// Gets all available monitors.
pub fn available_monitors() -> anyhow::Result<Vec<NativeMonitor>> {
  Ok(
    available_monitor_handles()?
      .into_iter()
      .map(NativeMonitor::new)
      .collect(),
  )
}

/// Gets all available monitor handles.
fn available_monitor_handles() -> anyhow::Result<Vec<isize>> {
  let mut monitors: Vec<isize> = Vec::new();

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      None,
      Some(available_monitor_handles_proc),
      LPARAM(std::ptr::from_mut(&mut monitors) as _),
    )
  }
  .ok()?;

  Ok(monitors)
}

/// Callback passed to `EnumDisplayMonitors` to get all available monitor
/// handles.
extern "system" fn available_monitor_handles_proc(
  handle: HMONITOR,
  _hdc: HDC,
  _clip: *mut RECT,
  data: LPARAM,
) -> BOOL {
  let handles = data.0 as *mut Vec<isize>;
  unsafe { (*handles).push(handle.0) };
  true.into()
}

#[must_use]
pub fn nearest_monitor(window_handle: isize) -> NativeMonitor {
  let handle = unsafe {
    MonitorFromWindow(HWND(window_handle), MONITOR_DEFAULTTONEAREST)
  };

  NativeMonitor::new(handle.0)
}

fn monitor_dpi(handle: isize) -> anyhow::Result<u32> {
  let mut dpi_x = u32::default();
  let mut dpi_y = u32::default();

  unsafe {
    GetDpiForMonitor(
      HMONITOR(handle),
      MDT_EFFECTIVE_DPI,
      &mut dpi_x,
      &mut dpi_y,
    )
  }?;

  // Arbitrarily choose the Y DPI.
  Ok(dpi_y)
}
