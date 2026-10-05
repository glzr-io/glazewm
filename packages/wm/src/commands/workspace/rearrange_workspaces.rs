use wm_common::WmEvent;

use super::{deactivate_workspace, sort_workspaces};
use crate::{
  commands::container::set_focused_descendant,
  models::{Monitor, Workspace},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Drops empty workspaces and renames the rest so that they hold the
/// leading configured workspace names in order, closing any gaps in the
/// numbering.
///
/// With workspaces `2`, `3`, `5`, `6` and `7` active, for instance, they
/// become `1`, `2`, `3`, `4` and `5`; with `1` to `4` active but `2`
/// empty, they become `1`, `2` and `3`. Windows are not moved: only the
/// workspaces' identity changes, so layouts are untouched.
///
/// Workspaces are ordered by monitor and then by their position in the
/// user config, so names stay grouped per monitor.
pub fn rearrange_workspaces(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  for monitor in state.monitors() {
    remove_empty_workspaces(&monitor, state, config)?;
  }

  let workspaces = ordered_workspaces(state, config);

  let current_names = workspaces
    .iter()
    .map(|workspace| workspace.config().name)
    .collect::<Vec<_>>();

  let configured_names = config
    .value
    .workspaces
    .iter()
    .map(|workspace_config| workspace_config.name.clone())
    .collect::<Vec<_>>();

  let plan = rearrange_plan(&current_names, &configured_names);

  if plan.iter().all(Option::is_none) {
    return Ok(());
  }

  // Resolve the recent workspace up front, since its name is about to
  // change and `general.toggle_workspace_on_refocus` looks it up by name.
  let recent_workspace = state
    .recent_workspace_name
    .clone()
    .and_then(|name| state.workspace_by_name(&name));

  let renamed = workspaces
    .iter()
    .zip(plan)
    .filter_map(|(workspace, config_index)| {
      Some((workspace, config.value.workspaces.get(config_index?)?))
    })
    .collect::<Vec<_>>();

  // Applied in full before any event is emitted, since a workspace can
  // briefly share a name with one that hasn't been renamed yet.
  for (workspace, workspace_config) in &renamed {
    // The whole config is adopted rather than just the name, so that
    // options such as `bind_to_monitor` and `display_name` follow the
    // name instead of the workspace that happened to hold it.
    workspace.set_config((*workspace_config).clone());
  }

  for (workspace, _) in &renamed {
    state.emit_event(WmEvent::WorkspaceUpdated {
      updated_workspace: workspace.to_dto()?,
    });
  }

  state.recent_workspace_name =
    recent_workspace.map(|workspace| workspace.config().name);

  for monitor in state.monitors() {
    sort_workspaces(&monitor, config)?;
  }

  Ok(())
}

/// Deactivates a monitor's workspaces that have no windows, so that they
/// don't hold on to a number once the numbering closes up.
///
/// A monitor always has to display a workspace, so an empty displayed
/// workspace is only dropped when the monitor has another one to show in
/// its place. The monitor then switches to the workspace that takes over
/// the empty one's position, so the user stays at the same number.
/// Workspaces with `keep_alive` are always kept.
fn remove_empty_workspaces(
  monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let mut workspaces = monitor.workspaces();
  config.sort_workspaces(&mut workspaces);

  let removable = workspaces
    .iter()
    .map(|workspace| {
      !workspace.config().keep_alive && !workspace.has_children()
    })
    .collect::<Vec<_>>();

  let displayed_index =
    monitor.displayed_workspace().and_then(|displayed| {
      workspaces
        .iter()
        .position(|workspace| workspace.id() == displayed.id())
    });

  if let Some(displayed_index) =
    displayed_index.filter(|&index| removable[index])
  {
    if let Some(replacement_index) =
      replacement_index(displayed_index, &removable)
    {
      show_in_place_of(
        &workspaces[displayed_index],
        &workspaces[replacement_index],
        state,
      );
    }
  }

  for (workspace, is_removable) in workspaces.into_iter().zip(removable) {
    if is_removable && !workspace.is_displayed() {
      deactivate_workspace(workspace, state)?;
    }
  }

  Ok(())
}

/// Displays `replacement` on its monitor in place of `displayed`.
///
/// Focus follows only if `displayed` held it, so that tidying up another
/// monitor doesn't pull focus away from the one being worked on.
fn show_in_place_of(
  displayed: &Workspace,
  replacement: &Workspace,
  state: &mut WmState,
) {
  let is_focused = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .is_some_and(|focused| focused.id() == displayed.id());

  if is_focused {
    let container_to_focus = replacement
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| replacement.clone().into());

    set_focused_descendant(&container_to_focus, None);
    state.pending_sync.queue_focus_change();
  } else {
    // Stopping at the workspace changes which workspace its monitor
    // displays without touching which monitor has focus.
    let replacement = replacement.clone().into();
    set_focused_descendant(&replacement, Some(&replacement));
  }

  state
    .pending_sync
    .queue_container_to_redraw(replacement.clone());
}

/// Position of the workspace to display in place of the empty displayed
/// workspace at `displayed_index`, given which of the monitor's
/// workspaces are about to be removed.
///
/// Prefers the next workspace that is kept, since that is the one taking
/// over the empty workspace's number, and falls back to the previous one.
/// Returns `None` when every workspace on the monitor is being removed.
fn replacement_index(
  displayed_index: usize,
  removable: &[bool],
) -> Option<usize> {
  let is_kept = |index: &usize| !removable[*index];

  (displayed_index + 1..removable.len())
    .find(is_kept)
    .or_else(|| (0..displayed_index).rev().find(is_kept))
}

/// Active workspaces ordered by monitor, then by their position in the
/// user config.
fn ordered_workspaces(
  state: &WmState,
  config: &UserConfig,
) -> Vec<Workspace> {
  state
    .monitors()
    .iter()
    .flat_map(|monitor| {
      let mut workspaces = monitor.workspaces();
      config.sort_workspaces(&mut workspaces);
      workspaces
    })
    .collect()
}

/// Index of the configured name that each workspace should take, given
/// the workspaces' current names in order.
///
/// An entry is `None` where the workspace already holds the right name,
/// or where there are more workspaces than configured names, in which
/// case the surplus keeps what it has.
fn rearrange_plan(
  current_names: &[String],
  configured_names: &[String],
) -> Vec<Option<usize>> {
  current_names
    .iter()
    .enumerate()
    .map(|(index, current_name)| {
      configured_names
        .get(index)
        .filter(|target_name| *target_name != current_name)
        .map(|_| index)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{rearrange_plan, replacement_index};

  #[test]
  fn replaces_an_empty_workspace_with_the_one_taking_its_number() {
    // Workspaces 1 to 4, with 2 displayed and empty: 3 becomes the new 2.
    assert_eq!(
      replacement_index(1, &[false, true, false, false]),
      Some(2)
    );
  }

  #[test]
  fn skips_other_empty_workspaces_when_replacing() {
    assert_eq!(replacement_index(0, &[true, true, false]), Some(2));
  }

  #[test]
  fn falls_back_to_the_previous_workspace_at_the_end() {
    assert_eq!(replacement_index(2, &[false, true, true]), Some(0));
  }

  #[test]
  fn keeps_the_only_workspace_when_everything_is_empty() {
    assert_eq!(replacement_index(0, &[true]), None);
    assert_eq!(replacement_index(1, &[true, true]), None);
  }

  fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
  }

  #[test]
  fn closes_gaps_in_the_numbering() {
    let configured = names(&["1", "2", "3", "4", "5", "6", "7", "8", "9"]);
    let current = names(&["2", "3", "5", "6", "7"]);

    assert_eq!(
      rearrange_plan(&current, &configured),
      vec![Some(0), Some(1), Some(2), Some(3), Some(4)]
    );
  }

  #[test]
  fn leaves_already_ordered_workspaces_alone() {
    let configured = names(&["1", "2", "3"]);
    let current = names(&["1", "2", "3"]);

    assert_eq!(
      rearrange_plan(&current, &configured),
      vec![None, None, None]
    );
  }

  #[test]
  fn renames_only_the_workspaces_that_are_out_of_place() {
    let configured = names(&["1", "2", "3"]);
    let current = names(&["1", "3"]);

    assert_eq!(rearrange_plan(&current, &configured), vec![None, Some(1)]);
  }

  #[test]
  fn keeps_workspaces_beyond_the_configured_names() {
    let configured = names(&["1"]);
    let current = names(&["2", "3"]);

    assert_eq!(rearrange_plan(&current, &configured), vec![Some(0), None]);
  }

  #[test]
  fn handles_names_that_are_not_numbers() {
    let configured = names(&["web", "code", "chat"]);
    let current = names(&["code", "chat"]);

    assert_eq!(
      rearrange_plan(&current, &configured),
      vec![Some(0), Some(1)]
    );
  }
}
