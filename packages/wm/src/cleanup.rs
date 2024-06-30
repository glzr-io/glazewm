use tracing::{info, warn};

use crate::common::platform::NativeWindow;

pub fn run_cleanup(managed_windows: Vec<NativeWindow>) {
  info!("Running WM state cleanup.",);

  for window in managed_windows {
    if let Err(err) = window.show() {
      warn!("Failed to show window: {:?}", err);
    }

    _ = window.set_border_color(None);
  }
}
