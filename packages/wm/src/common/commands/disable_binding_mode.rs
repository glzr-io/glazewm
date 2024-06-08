use crate::{wm_event::WmEvent, wm_state::WmState};

pub fn disable_binding_mode(name: &str, state: &mut WmState) {
  state.binding_modes = state
    .binding_modes
    .iter()
    .filter(|config| config.name != name)
    .cloned()
    .collect::<Vec<_>>();

  state.emit_event(WmEvent::BindingModesChanged {
    active_binding_modes: state.binding_modes.clone(),
  });
}
