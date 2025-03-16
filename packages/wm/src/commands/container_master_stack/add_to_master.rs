use crate::{
  models::{TilingLayout, TilingWindow, Workspace},
  traits::CommonGetters,
};

pub fn add_to_master(
  window: TilingWindow,
  workspace: &Workspace,
) -> anyhow::Result<()> {
  tracing::info!("Adding window to master stack layout.");
  match workspace.tiling_layout() {
    TilingLayout::MasterStack { master_window } => {
      // Remove the window from the workspace's children (the stack)
      workspace
        .borrow_children_mut()
        .retain(|child| child.id() != window.id());
      // Set the new master window
      // TODO - swap the old one back into children
      workspace.set_tiling_layout(TilingLayout::MasterStack {
        master_window: Some(window)
      });
      Ok(())
    }
    _ => Err(anyhow::anyhow!(
      "Workspace is not using a MasterStack layout."
    )),
  }
}
