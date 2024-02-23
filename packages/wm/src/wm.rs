use wineventhook::WindowEvent;

use crate::{user_config::UserConfig, wm_state::WmState};

pub struct WindowManager {
  state: WmState,
}

impl WindowManager {
  pub fn new(user_config: UserConfig) -> Self {
    Self {
      state: WmState::new(user_config),
    }
  }

  pub fn init(&mut self) {
    todo!()
  }

  pub fn process_event(event: WindowEvent) {
    todo!()
  }
}
