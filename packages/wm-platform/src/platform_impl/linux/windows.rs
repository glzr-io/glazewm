use smithay::wayland::shell::xdg::ToplevelSurface;

use super::NativeWindow;

#[derive(Debug, Default)]
pub struct Windows {
  pub mapped: Vec<NativeWindow>,
  pub unmapped: Vec<NativeWindow>,
}

impl Windows {
  pub fn new_window(&mut self, window: NativeWindow) -> &NativeWindow {
    self.unmapped.push(window);
    self.unmapped.last().unwrap()
  }

  pub fn window_close(&mut self, surface: &ToplevelSurface) {
    let idx = self
      .unmapped
      .iter()
      .enumerate()
      .find(|(_, w)| w.toplevel().is_some_and(|s| *s == *surface))
      .map(|(i, _)| i);
    if let Some(idx) = idx {
      self.unmapped.remove(idx);
    }

    let idx = self
      .mapped
      .iter()
      .enumerate()
      .find(|(_, w)| w.toplevel().is_some_and(|s| *s == *surface))
      .map(|(i, _)| i);
    if let Some(idx) = idx {
      self.mapped.remove(idx);
    }
  }
}
