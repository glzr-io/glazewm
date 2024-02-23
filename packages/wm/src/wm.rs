pub struct WindowManager {
  state: WmState,
}

impl WindowManager {
  pub fn new() -> Self {
    Self {
      state: WmState::new(),
    }
  }

  pub fn init(&mut self) {
    todo!()
  }

  pub fn process_event(event: WindowEvent) {
    todo!()
  }
}
