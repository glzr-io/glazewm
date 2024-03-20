use windows::Win32::{
  Foundation::{BOOL, LPARAM, RECT},
  Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
  },
};

pub type MonitorHandle = HMONITOR;

#[derive(Debug)]
pub struct NativeMonitor {
  pub handle: MonitorHandle,
  pub device_name: String,
  pub width: i32,
  pub height: i32,
  pub x: i32,
  pub y: i32,
}

impl NativeMonitor {
  pub fn new(
    handle: MonitorHandle,
    device_name: String,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
  ) -> Self {
    Self {
      handle,
      device_name,
      width,
      height,
      x,
      y,
    }
  }
}

pub fn available_monitors() -> Vec<NativeMonitor> {
  available_monitor_handles()
    .into_iter()
    .filter_map(|handle| handle_to_monitor(handle).ok())
    .collect()
}

/// Gets all available monitor handles.
fn available_monitor_handles() -> Vec<MonitorHandle> {
  let mut monitors: Vec<MonitorHandle> = Vec::new();

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      None,
      Some(available_monitor_handles_proc),
      LPARAM(&mut monitors as *mut _ as _),
    );
  }

  monitors
}

/// Callback passed to `EnumDisplayMonitors` to get all available monitor
/// handles.
unsafe extern "system" fn available_monitor_handles_proc(
  handle: MonitorHandle,
  _hdc: HDC,
  _clip: *mut RECT,
  data: LPARAM,
) -> BOOL {
  let monitors = data.0 as *mut Vec<MonitorHandle>;
  unsafe { (*monitors).push(handle) };
  true.into()
}

/// Converts a monitor handle to an instance of `NativeMonitor`.
fn handle_to_monitor(
  handle: MonitorHandle,
) -> anyhow::Result<NativeMonitor> {
  let mut monitor_info = MONITORINFOEXW::default();
  monitor_info.monitorInfo.cbSize =
    std::mem::size_of::<MONITORINFOEXW>() as u32;

  unsafe {
    GetMonitorInfoW(handle, &mut monitor_info as *mut _ as *mut _)
  }
  .ok()?;

  let device_name = String::from_utf16_lossy(&monitor_info.szDevice);

  Ok(NativeMonitor::new(
    handle,
    device_name,
    monitor_info.monitorInfo.rcMonitor.right
      - monitor_info.monitorInfo.rcMonitor.left,
    monitor_info.monitorInfo.rcMonitor.bottom
      - monitor_info.monitorInfo.rcMonitor.top,
    monitor_info.monitorInfo.rcMonitor.left,
    monitor_info.monitorInfo.rcMonitor.top,
  ))
}
