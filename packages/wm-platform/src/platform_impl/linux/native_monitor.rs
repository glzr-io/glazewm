use smithay::{
  output::Output,
  utils::{Physical, Size},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMonitor {
  inner: Output,
}

impl NativeMonitor {
  #[must_use]
  pub fn handle(&self) -> &Output {
    &self.inner
  }

  pub fn device_name(&self) -> anyhow::Result<String> {
    Ok(self.inner.name())
  }

  pub fn device_path(&self) -> anyhow::Result<Option<&String>> {
    todo!()
  }

  pub fn hardware_id(&self) -> anyhow::Result<Option<&String>> {
    todo!()
  }

  pub fn working_rect(&self) -> anyhow::Result<wm_common::Rect> {
    let pos = self.inner.current_location();
    let size = self
      .size()
      .ok_or_else(|| anyhow::anyhow!("Monitor has no available size"))?;

    Ok(wm_common::Rect::from_xy(pos.x, pos.y, size.w, size.h))
  }

  fn size(&self) -> Option<Size<i32, Physical>> {
    // Prefer current
    let size = self.inner.current_mode().map(|mode| mode.size);
    if size.is_some() {
      return size;
    }

    // Else what the monitor likes
    let size = self.inner.preferred_mode().map(|mode| mode.size);
    if size.is_some() {
      return size;
    }

    // Otherwise, fallback to whatever is available
    // TODO: Can probably be smarter with this, if needed
    self.inner.modes().first().map(|mode| mode.size)
  }
}
