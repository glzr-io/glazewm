use crate::{
  models::{TilingLayout, TilingWindow, Workspace},
  traits::CommonGetters,
  wm_state::WmState,
};

pub fn add_to_master(
  window: TilingWindow,
  workspace: &Workspace,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match workspace.tiling_layout() {
    TilingLayout::MasterStack { master_window } => {
      match master_window {
        None => {
          // Since there are now less windows in the stack, all stack
          // children need resized/redrawn.
          state
            .pending_sync
            .queue_containers_to_redraw(workspace.tiling_children());
        }
        Some(old_master_window) => {
          // Swap the old master_window with one of the stack windows
          let old_master_container = old_master_window.as_container();
          let index = window.index();
          workspace
            .borrow_children_mut()
            .insert(index, old_master_container.clone());
          state
            .pending_sync
            .queue_container_to_redraw(old_master_container);
        }
      }

      // Set the new master window
      workspace.set_tiling_layout(TilingLayout::MasterStack {
        master_window: Some(window.clone()),
      });

      // Remove the window from the workspace's children (the stack).
      println!(
        "length of children: {}",
        workspace.borrow_children().len()
      );
      workspace
        .borrow_children_mut()
        .retain(|child| child.id() != window.id());
      println!(
        "length of children: {}",
        workspace.borrow_children().len()
      );
      state.pending_sync.queue_container_to_redraw(window.clone());
      Ok(())
    }
    _ => Err(anyhow::anyhow!(
      "Workspace is not using a MasterStack layout."
    )),
  }
}
