use crate::{
  containers::{commands::detach_container, traits::CommonGetters},
  wm_event::WmEvent,
  wm_state::WmState,
  workspaces::Workspace,
};

/// Deactivates a given workspace. This removes the container from its
/// parent monitor and emits a `WorkspaceDeactivated` event.
pub fn deactivate_workspace(
  workspace: Workspace,
  state: &WmState,
) -> anyhow::Result<()> {
  if state.paused {
    return Ok(());
  }

  detach_container(workspace.clone().into())?;

  state.emit_event(WmEvent::WorkspaceDeactivated {
    deactivated_id: workspace.id(),
    deactivated_name: workspace.config().name,
  });

  Ok(())
}
