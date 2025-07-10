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

  pub fn find_from_surface(
    &self,
    surface: &ToplevelSurface,
  ) -> Option<&NativeWindow> {
    let mapped = self
      .mapped
      .iter()
      .find(|w| w.toplevel().is_some_and(|s| *s == *surface));
    if mapped.is_some() {
      return mapped;
    }

    self
      .unmapped
      .iter()
      .find(|w| w.toplevel().is_some_and(|s| *s == *surface))
  }

  pub fn find_from_surface_mut(
    &mut self,
    surface: &ToplevelSurface,
  ) -> Option<&mut NativeWindow> {
    let mapped = self
      .mapped
      .iter_mut()
      .find(|w| w.toplevel().is_some_and(|s| *s == *surface));
    if mapped.is_some() {
      return mapped;
    }

    self
      .unmapped
      .iter_mut()
      .find(|w| w.toplevel().is_some_and(|s| *s == *surface))
  }
}
