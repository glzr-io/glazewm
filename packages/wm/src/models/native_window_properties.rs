use wm_platform::{NativeWindow, Rect};

use crate::{models::Workspace, traits::PositionGetters};

#[derive(Debug, Clone)]
pub struct NativeWindowProperties {
  pub title: String,
  pub class_name: String,
  pub process_name: String,
  pub frame: Rect,
  pub is_minimized: bool,
  pub is_maximized: bool,
  pub is_fullscreen: bool,
}

impl NativeWindowProperties {
  pub fn try_from(
    native_window: &NativeWindow,
    nearest_workspace: &Workspace,
  ) -> anyhow::Result<Self> {
    Ok(Self {
      title: native_window.title()?,
      class_name: native_window.class_name()?,
      process_name: native_window.process_name()?,
      frame: native_window.frame()?,
      is_minimized: native_window.is_minimized()?,
      is_maximized: native_window.is_maximized()?,
      is_fullscreen: native_window
        .is_fullscreen(&nearest_workspace.to_rect()?)?,
    })
  }
}
