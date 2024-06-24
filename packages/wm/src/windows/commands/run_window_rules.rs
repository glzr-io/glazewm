use tracing::info;

use crate::{
  containers::{traits::CommonGetters, WindowContainer},
  user_config::{UserConfig, WindowRuleEvent},
  windows::traits::WindowGetters,
  wm_state::WmState,
};

/// Returns the window (if it's still attached) after running the window
/// rules.
pub fn run_window_rules(
  window: WindowContainer,
  event_type: WindowRuleEvent,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<Option<WindowContainer>> {
  let pending_window_rules =
    config.pending_window_rules(&window, &event_type)?;

  let mut subject_window = window;

  for rule in pending_window_rules {
    info!("Running window rule with commands: {:?}.", rule.commands);

    for command in &rule.commands {
      command.run(subject_window.clone().into(), state, config)?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      subject_window = match subject_window.is_detached() {
        false => subject_window,
        true => match state.window_from_native(&subject_window.native()) {
          Some(window) => window,
          None => return Ok(None),
        },
      }
    }

    // Add the window rule as done.
    if rule.run_once {
      let window_rules = subject_window
        .done_window_rules()
        .into_iter()
        .chain(std::iter::once(rule));

      subject_window.set_done_window_rules(window_rules.collect());
    }
  }

  Ok(Some(subject_window))
}
