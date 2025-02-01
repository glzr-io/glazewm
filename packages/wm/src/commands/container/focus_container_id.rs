use uuid::Uuid;

use super::set_focused_descendant;
use crate::wm_state::WmState;

pub fn focus_container_id(
  container_id: &Uuid,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let focus_target = state.container_by_id(*container_id);

  // Set focus to the target container.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  }

  Ok(())
}
