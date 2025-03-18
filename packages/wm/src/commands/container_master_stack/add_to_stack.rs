use crate::{
  models::{TilingLayout, TilingWindow, Workspace},
  traits::CommonGetters,
  wm_state::WmState,
};

pub fn add_to_stack(
  window: TilingWindow,
  workspace: &Workspace,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match workspace.tiling_layout() {
    TilingLayout::MasterStack { master_window } => {
      match master_window {
        Some(master) => {
          // Add the window to the stack.
          workspace.borrow_children_mut().push_front(window.as_container());
          // state.pending_sync.queue_container_to_redraw(window.clone());
          // Resize all stack windows to make room for new window
          state
            .pending_sync
            .queue_containers_to_redraw(workspace.tiling_children());
          Ok(())
        }
        None => todo!(),
      }
    }
    _ => Err(anyhow::anyhow!(
      "Workspace is not using a MasterStack layout."
    )),
  }
}
