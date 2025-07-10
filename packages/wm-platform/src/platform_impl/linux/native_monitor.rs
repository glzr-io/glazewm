use smithay::{
  desktop::{Space, Window},
  reexports::{wayland_server::Display, winit::monitor::MonitorHandle},
};

use super::state::Glaze;

pub struct NativeMonitor {
  inner: MonitorHandle,
}

impl NativeMonitor {
  pub fn device_name(&self) -> anyhow::Result<String> {
    self
      .inner
      .name()
      .ok_or_else(|| anyhow::anyhow!("Monitor name not available"))
  }

  pub fn working_rect(&self) -> anyhow::Result<wm_common::Rect> {
    let pos = self.inner.position();
    let size = self.inner.size();

    Ok(wm_common::Rect::from_xy(
      pos.x,
      pos.y,
      size.width as i32,
      size.height as i32,
    ))
  }
}
