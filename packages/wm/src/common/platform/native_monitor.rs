use windows::Win32::{
  Foundation::{BOOL, LPARAM, RECT},
  Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
  },
};

pub type MonitorHandle = HMONITOR;

// TODO: Consider changing `device_name`, `width`, `height`, `x`, and `y` to
// be lazily retrieved similar to in `NativeWindow`. Add an `refresh` method
// to `NativeMonitor` to refresh the values.
#[derive(Clone, Debug)]
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

impl PartialEq for NativeMonitor {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeMonitor {}

pub fn available_monitors() -> anyhow::Result<Vec<NativeMonitor>> {
  Ok(
    available_monitor_handles()?
      .into_iter()
      .filter_map(|handle| handle_to_monitor(handle).ok())
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
unsafe extern "system" fn available_monitor_handles_proc(
  handle: MonitorHandle,
  _hdc: HDC,
  _clip: *mut RECT,
  data: LPARAM,
) -> BOOL {
  let handles = data.0 as *mut Vec<MonitorHandle>;
  unsafe { (*handles).push(handle) };
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
