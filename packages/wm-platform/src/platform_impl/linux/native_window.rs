use smithay::desktop::Window;
use wm_common::{
  Color, CornerStyle, HideMethod, OpacityValue, Rect, WindowState,
};

use crate::ZOrder;

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWindow {
  // Smithay doesn't expose a window ID (that I can find), so create our
  // own
  id: uuid::Uuid,
  inner: Window,
}

impl NativeWindow {
  #[must_use]
  pub fn new(inner: Window) -> Self {
    let id = uuid::Uuid::new_v4();
    Self { id, inner }
  }

  #[must_use]
  pub fn handle(&self) -> crate::WindowHandle {
    self.id
  }

  pub fn frame_position(&self) -> anyhow::Result<Rect> {
    // Assuming the frame position is the same as the window position
    let geom = self.inner.geometry();
    let rect =
      Rect::from_xy(geom.loc.x, geom.loc.y, geom.size.w, geom.size.h);
    Ok(rect)
  }

  pub fn is_minimized(&self) -> anyhow::Result<bool> {
    todo!()
  }

  pub fn minimize(&self) -> anyhow::Result<()> {
    todo!()
  }

  pub fn is_maximized(&self) -> anyhow::Result<bool> {
    todo!()
  }

  pub fn mark_fullscreen(&self, b: bool) -> anyhow::Result<()> {
    todo!()
  }

  pub fn is_fullscreen(&self, rect: &Rect) -> anyhow::Result<bool> {
    todo!()
  }

  #[must_use]
  pub fn is_resizable(&self) -> bool {
    todo!()
  }

  pub fn set_taskbar_visibility(&self, b: bool) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_border_color(
    &self,
    color: Option<&Color>,
  ) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_title_bar_visibility(&self, b: bool) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_corner_style(&self, b: &CornerStyle) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_transparency(&self, v: &OpacityValue) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_foreground(&self) -> anyhow::Result<()> {
    todo!()
  }

  pub fn show(&self) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_position(
    &self,
    state: &WindowState,
    rect: &Rect,
    z_order: &ZOrder,
    is_visible: bool,
    hide_method: &HideMethod,
    has_pending_dpi_adjustment: bool,
  ) -> anyhow::Result<()> {
    todo!()
  }

  pub fn set_z_order(&self, _z_order: &ZOrder) -> anyhow::Result<()> {
    todo!()
  }
}

impl std::ops::Deref for NativeWindow {
  type Target = Window;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl std::ops::DerefMut for NativeWindow {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
