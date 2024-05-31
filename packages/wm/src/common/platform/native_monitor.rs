use std::cell::OnceCell;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
      EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW,
      MonitorFromWindow, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, HDC,
      HMONITOR, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    UI::{
      HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
      WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
  },
};

use crate::common::Rect;

pub type MonitorHandle = HMONITOR;

#[derive(Clone, Debug)]
pub struct NativeMonitor {
  pub handle: MonitorHandle,
  info: OnceCell<MonitorInfo>,
}

#[derive(Clone, Debug)]
struct MonitorInfo {
  device_name: String,
  device_path: Option<String>,
  hardware_id: Option<String>,
  rect: Rect,
  working_rect: Rect,
  dpi: f32,
}

impl NativeMonitor {
  pub fn new(handle: MonitorHandle) -> Self {
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

  pub fn dpi(&self) -> anyhow::Result<f32> {
    self.monitor_info().map(|info| info.dpi)
  }

  fn monitor_info(&self) -> anyhow::Result<&MonitorInfo> {
    self.info.get_or_try_init(|| {
      let mut monitor_info = MONITORINFOEXW::default();
      monitor_info.monitorInfo.cbSize =
        std::mem::size_of::<MONITORINFOEXW>() as u32;

      unsafe {
        GetMonitorInfoW(self.handle, &mut monitor_info as *mut _ as _)
      }
      .ok()?;

      // Get the display devices associated with the monitor.
      let mut display_devices = (0..)
        .map_while(|index| {
          let mut display_device = DISPLAY_DEVICEW::default();
          display_device.cb =
            std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

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
          .then(|| display_device)
        })
        // Filter out any devices that are not active.
        .filter(|device| device.StateFlags & DISPLAY_DEVICE_ACTIVE != 0);

      // Get the device path and hardware ID from the first valid device.
      let (device_path, hardware_id) = display_devices
        .next()
        .map(|device| {
          let device_path = String::from_utf16_lossy(&device.DeviceID)
            .trim_end_matches('\0')
            .to_string();

          let hardware_id = device_path
            .split("#")
            .collect::<Vec<_>>()
            .get(1)
            .map(|id| id.to_string());

          (Some(device_path), hardware_id)
        })
        .unwrap_or((None, None));

      let device_name = String::from_utf16_lossy(&monitor_info.szDevice);
      let dpi = monitor_dpi(self.handle)?;

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
      .map(|handle| NativeMonitor::new(handle))
      .collect(),
  )
}

/// Gets all available monitor handles.
fn available_monitor_handles() -> anyhow::Result<Vec<MonitorHandle>> {
  let mut monitors: Vec<MonitorHandle> = Vec::new();

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      None,
      Some(available_monitor_handles_proc),
      LPARAM(&mut monitors as *mut _ as _),
    )
  }
  .ok()?;

  Ok(monitors)
}

/// Callback passed to `EnumDisplayMonitors` to get all available monitor
/// handles.
extern "system" fn available_monitor_handles_proc(
  handle: MonitorHandle,
  _hdc: HDC,
  _clip: *mut RECT,
  data: LPARAM,
) -> BOOL {
  let handles = data.0 as *mut Vec<MonitorHandle>;
  unsafe { (*handles).push(handle) };
  true.into()
}

pub fn nearest_monitor(window_handle: isize) -> NativeMonitor {
  let handle = unsafe {
    MonitorFromWindow(HWND(window_handle), MONITOR_DEFAULTTONEAREST)
  };

  NativeMonitor::new(handle)
}

fn monitor_dpi(handle: MonitorHandle) -> anyhow::Result<f32> {
  let mut dpi_x = u32::default();
  let mut dpi_y = u32::default();

  unsafe {
    GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y)
  }?;

  Ok(dpi_y as f32 / 96.0)
}
