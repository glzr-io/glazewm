use std::collections::BTreeMap;

use uuid::Uuid;

/// A managed window of a grouped application, captured before or after
/// a command or platform event.
#[derive(Clone, Debug)]
pub struct GroupedWindow {
  /// ID of the window container.
  pub id: Uuid,

  /// Bundle ID of the window's application.
  pub application: String,

  /// ID of the workspace containing the window.
  pub workspace: Uuid,

  /// Whether the window is the WM's focused container.
  pub has_focus: bool,
}

/// Plans the moves that keep each grouped application on a single
/// workspace.
///
/// The group follows a window that was explicitly moved between `before`
/// and `after`. Without an explicit move, new windows (e.g. native tabs)
/// inherit the workspace of an existing member instead of relocating
/// the group. Focus changes alone never move a group.
///
/// Returns a vector of `(window_id, workspace_id)` pairs.
pub fn plan_group_moves(
  before: &[GroupedWindow],
  after: &[GroupedWindow],
) -> Vec<(Uuid, Uuid)> {
  let mut groups: BTreeMap<&str, Vec<&GroupedWindow>> = BTreeMap::new();
  for window in after {
    groups.entry(&window.application).or_default().push(window);
  }

  let mut moves = Vec::new();

  for windows in groups.values() {
    let was_moved = |window: &&GroupedWindow| {
      before.iter().any(|old| {
        old.id == window.id
          && old.application == window.application
          && old.workspace != window.workspace
      })
    };

    let existed_before = |window: &&GroupedWindow| {
      before.iter().any(|old| {
        old.id == window.id && old.application == window.application
      })
    };

    // Prefer an explicitly moved window, then an existing member, and
    // finally any member (e.g. on startup).
    let anchor = windows
      .iter()
      .copied()
      .filter(was_moved)
      .find(|window| window.has_focus)
      .or_else(|| windows.iter().copied().find(was_moved))
      .or_else(|| {
        windows
          .iter()
          .copied()
          .filter(existed_before)
          .find(|window| window.has_focus)
      })
      .or_else(|| windows.iter().copied().find(existed_before))
      .or_else(|| windows.iter().copied().find(|window| window.has_focus))
      .or_else(|| windows.first().copied());

    if let Some(anchor) = anchor {
      moves.extend(
        windows
          .iter()
          .filter(|window| window.workspace != anchor.workspace)
          .map(|window| (window.id, anchor.workspace)),
      );
    }
  }

  moves
}

#[cfg(test)]
mod tests {
  use super::*;

  fn window(
    id: u128,
    application: &str,
    workspace: u128,
    has_focus: bool,
  ) -> GroupedWindow {
    GroupedWindow {
      id: Uuid::from_u128(id),
      application: application.into(),
      workspace: Uuid::from_u128(workspace),
      has_focus,
    }
  }

  #[test]
  fn new_tab_inherits_existing_workspace_even_when_it_has_focus() {
    let old = window(1, "ghostty", 10, false);
    let new_tab = window(2, "ghostty", 20, true);
    assert_eq!(
      plan_group_moves(
        std::slice::from_ref(&old),
        &[old.clone(), new_tab]
      ),
      vec![(Uuid::from_u128(2), Uuid::from_u128(10))]
    );
  }

  #[test]
  fn moving_one_window_moves_its_group_without_pinning_a_workspace() {
    let before = vec![
      window(1, "ghostty", 10, true),
      window(2, "ghostty", 10, false),
    ];
    let after = vec![
      window(1, "ghostty", 20, false),
      window(2, "ghostty", 10, true),
    ];
    assert_eq!(
      plan_group_moves(&before, &after),
      vec![(Uuid::from_u128(2), Uuid::from_u128(20))]
    );

    let before = vec![
      window(1, "ghostty", 20, false),
      window(2, "ghostty", 20, true),
    ];
    let after = vec![
      window(1, "ghostty", 20, false),
      window(2, "ghostty", 30, false),
    ];
    assert_eq!(
      plan_group_moves(&before, &after),
      vec![(Uuid::from_u128(1), Uuid::from_u128(30))]
    );
  }

  #[test]
  fn closing_a_tab_does_not_move_surviving_windows() {
    let before = vec![
      window(1, "ghostty", 10, true),
      window(2, "ghostty", 10, false),
    ];
    assert_eq!(
      plan_group_moves(&before, &[before[1].clone()]),
      Vec::new()
    );
  }

  #[test]
  fn independent_application_groups_keep_their_own_workspaces() {
    let before = vec![
      window(1, "ghostty", 10, false),
      window(3, "other", 30, false),
    ];
    let after = vec![
      window(1, "ghostty", 20, false),
      window(2, "ghostty", 10, false),
      window(3, "other", 30, false),
    ];
    assert_eq!(
      plan_group_moves(&before, &after),
      vec![(Uuid::from_u128(2), Uuid::from_u128(20))]
    );
  }

  #[test]
  fn startup_consolidates_a_split_group_around_its_focused_window() {
    let after = vec![
      window(1, "ghostty", 10, false),
      window(2, "ghostty", 20, true),
    ];
    assert_eq!(
      plan_group_moves(&[], &after),
      vec![(Uuid::from_u128(1), Uuid::from_u128(20))]
    );
  }

  #[test]
  fn focus_changes_do_not_relocate_a_group() {
    let before = vec![
      window(1, "ghostty", 10, false),
      window(2, "ghostty", 10, false),
    ];
    let after = vec![
      window(1, "ghostty", 10, false),
      window(2, "ghostty", 10, true),
    ];
    assert_eq!(plan_group_moves(&before, &after), Vec::new());
  }

  #[test]
  fn every_remaining_tab_follows_a_background_window_move() {
    let before = vec![
      window(1, "ghostty", 10, true),
      window(2, "ghostty", 10, false),
      window(3, "ghostty", 10, false),
    ];
    let after = vec![
      window(1, "ghostty", 10, true),
      window(2, "ghostty", 20, false),
      window(3, "ghostty", 10, false),
    ];
    assert_eq!(
      plan_group_moves(&before, &after),
      vec![
        (Uuid::from_u128(1), Uuid::from_u128(20)),
        (Uuid::from_u128(3), Uuid::from_u128(20))
      ]
    );
  }
}
