#[derive(Debug)]
pub struct EventWindow {
  keyboard_hook: Arc<KeyboardHook>,
  window_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl EventWindow {
  pub fn update(
    &mut self,
    keybindings: &Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) {
    self.keyboard_hook.update(keybindings);
    ENABLE_MOUSE_EVENTS.store(enable_mouse_events, Ordering::Relaxed);
  }
}
