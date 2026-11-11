use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{GapsConfig, GroupsConfig, ParsedConfig};
use wm_platform::{Direction, Dispatcher, Rect};

use super::{
  create_group, cycle_group_page, insert_window_into_group,
  move_window_in_group, ungroup_window,
};
use crate::{
  commands::container::{
    attach_container, detach_container, set_focused_descendant,
  },
  models::{
    is_on_inactive_page, GroupContainer, Monitor, TilingContainer,
    TilingWindow, Workspace,
  },
  traits::{CommonGetters, PositionGetters},
  wm_state::WmState,
};

fn create_state() -> WmState {
  let (event_tx, _event_rx) = mpsc::unbounded_channel();
  let (exit_tx, _exit_rx) = mpsc::unbounded_channel();

  WmState::new(Dispatcher::mock(), event_tx, exit_tx)
}

fn window() -> TilingWindow {
  TilingWindow::mock().call()
}

/// Creates a workspace with the given children, attached to a monitor.
///
/// The monitor is kept alive by the workspace's parent reference.
fn workspace_with(children: Vec<TilingContainer>) -> Workspace {
  let workspace = Workspace::mock().tiling_containers(children).call();
  let _ = Monitor::mock().workspaces(vec![workspace.clone()]).call();
  workspace
}

/// Creates a group with the given windows as pages, in order.
fn group_with(pages: &[&TilingWindow]) -> GroupContainer {
  let group =
    GroupContainer::new(GapsConfig::default(), GroupsConfig::default());

  for page in pages {
    attach_container(&(*page).clone().into(), &group.clone().into(), None)
      .unwrap();
  }

  group
}

fn page_ids(group: &GroupContainer) -> Vec<Uuid> {
  group.children().iter().map(CommonGetters::id).collect()
}

fn child_ids(container: &impl CommonGetters) -> Vec<Uuid> {
  container.children().iter().map(CommonGetters::id).collect()
}

fn active_id(group: &GroupContainer) -> Uuid {
  group.active_window().unwrap().id()
}

#[test]
fn create_group_wraps_window_at_its_position() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let workspace = workspace_with(vec![
    a.clone().into(),
    b.clone().into(),
    c.clone().into(),
  ]);

  create_group(&b, &ParsedConfig::default(), &mut state).unwrap();

  let group = workspace
    .children()
    .get(1)
    .and_then(|child| child.as_group().cloned())
    .expect("Expected a group at the window's position.");

  assert_eq!(child_ids(&workspace).len(), 3);
  assert_eq!(child_ids(&workspace)[0], a.id());
  assert_eq!(child_ids(&workspace)[2], c.id());
  assert_eq!(page_ids(&group), vec![b.id()]);
  assert_eq!(active_id(&group), b.id());
}

#[test]
fn create_group_is_noop_for_grouped_window() {
  let mut state = create_state();
  let (a, b) = (window(), window());
  let workspace = workspace_with(vec![a.clone().into(), b.clone().into()]);

  create_group(&a, &ParsedConfig::default(), &mut state).unwrap();
  create_group(&a, &ParsedConfig::default(), &mut state).unwrap();

  assert_eq!(workspace.child_count(), 2);

  let group = workspace.children()[0].as_group().cloned().unwrap();
  assert_eq!(page_ids(&group), vec![a.id()]);
}

#[test]
fn cycle_group_page_wraps_in_both_directions() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&a, &b, &c]);
  let _workspace = workspace_with(vec![group.clone().into()]);
  set_focused_descendant(&a.clone().into(), None);

  cycle_group_page(&a, true, &mut state).unwrap();
  assert_eq!(active_id(&group), b.id());

  cycle_group_page(&b, true, &mut state).unwrap();
  assert_eq!(active_id(&group), c.id());

  // Wraps from the last page to the first.
  cycle_group_page(&c, true, &mut state).unwrap();
  assert_eq!(active_id(&group), a.id());

  // Wraps from the first page to the last.
  cycle_group_page(&a, false, &mut state).unwrap();
  assert_eq!(active_id(&group), c.id());
}

#[test]
fn cycle_group_page_is_noop_for_single_page_group() {
  let mut state = create_state();
  let a = window();
  let group = group_with(&[&a]);
  let _workspace = workspace_with(vec![group.clone().into()]);

  cycle_group_page(&a, true, &mut state).unwrap();

  assert_eq!(active_id(&group), a.id());
}

#[test]
fn only_active_page_is_displayed() {
  let (a, b) = (window(), window());
  let group = group_with(&[&a, &b]);
  let _workspace = workspace_with(vec![group.clone().into()]);
  set_focused_descendant(&a.clone().into(), None);

  assert!(!is_on_inactive_page(&a));
  assert!(is_on_inactive_page(&b));

  set_focused_descendant(&b.clone().into(), None);

  assert!(is_on_inactive_page(&a));
  assert!(!is_on_inactive_page(&b));
}

#[test]
fn move_right_and_left_reorders_pages() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&a, &b, &c]);
  let _workspace = workspace_with(vec![group.clone().into()]);

  let completed =
    move_window_in_group(&b, &group, &Direction::Right, &mut state)
      .unwrap();

  assert!(completed);
  assert_eq!(page_ids(&group), vec![a.id(), c.id(), b.id()]);

  let completed =
    move_window_in_group(&b, &group, &Direction::Left, &mut state)
      .unwrap();

  assert!(completed);
  assert_eq!(page_ids(&group), vec![a.id(), b.id(), c.id()]);
}

#[test]
fn move_right_from_last_page_ejects_it_after_group() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&a, &b, &c]);
  let workspace = workspace_with(vec![group.clone().into()]);

  let completed =
    move_window_in_group(&c, &group, &Direction::Right, &mut state)
      .unwrap();

  assert!(completed);
  assert_eq!(page_ids(&group), vec![a.id(), b.id()]);
  assert_eq!(child_ids(&workspace), vec![group.id(), c.id()]);
}

#[test]
fn move_left_from_first_page_ejects_it_before_group() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&a, &b, &c]);
  let workspace = workspace_with(vec![group.clone().into()]);

  move_window_in_group(&a, &group, &Direction::Left, &mut state).unwrap();

  assert_eq!(page_ids(&group), vec![b.id(), c.id()]);
  assert_eq!(child_ids(&workspace), vec![a.id(), group.id()]);
}

#[test]
fn move_up_ejects_page_and_defers_to_tiling_logic() {
  let mut state = create_state();
  let (a, b) = (window(), window());
  let group = group_with(&[&a, &b]);
  let workspace = workspace_with(vec![group.clone().into()]);

  let completed =
    move_window_in_group(&b, &group, &Direction::Up, &mut state).unwrap();

  assert!(!completed);
  assert_eq!(page_ids(&group), vec![a.id()]);
  assert_eq!(child_ids(&workspace), vec![b.id(), group.id()]);
}

#[test]
fn moving_only_page_left_dissolves_group() {
  let mut state = create_state();
  let (x, a, y) = (window(), window(), window());
  let group = group_with(&[&a]);
  let workspace = workspace_with(vec![
    x.clone().into(),
    group.clone().into(),
    y.clone().into(),
  ]);

  let completed =
    move_window_in_group(&a, &group, &Direction::Left, &mut state)
      .unwrap();

  assert!(completed);
  assert_eq!(child_ids(&workspace), vec![x.id(), a.id(), y.id()]);
  assert!(group.is_detached());
  assert_eq!(a.parent().unwrap().id(), workspace.id());
}

#[test]
fn ungroup_removes_window_after_group_when_group_has_other_pages() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&a, &b, &c]);
  let workspace = workspace_with(vec![group.clone().into()]);

  ungroup_window(&b, &mut state).unwrap();

  assert_eq!(page_ids(&group), vec![a.id(), c.id()]);
  assert_eq!(child_ids(&workspace), vec![group.id(), b.id()]);
}

#[test]
fn ungroup_dissolves_group_with_only_window() {
  let mut state = create_state();
  let (x, a) = (window(), window());
  let group = group_with(&[&a]);
  let workspace =
    workspace_with(vec![x.clone().into(), group.clone().into()]);

  ungroup_window(&a, &mut state).unwrap();

  assert_eq!(child_ids(&workspace), vec![x.id(), a.id()]);
  assert!(group.is_detached());
}

#[test]
fn ungroup_is_noop_for_ungrouped_window() {
  let mut state = create_state();
  let (a, b) = (window(), window());
  let workspace = workspace_with(vec![a.clone().into(), b.clone().into()]);

  ungroup_window(&a, &mut state).unwrap();

  assert_eq!(child_ids(&workspace), vec![a.id(), b.id()]);
}

#[test]
fn inserted_window_becomes_page_after_active_page() {
  let mut state = create_state();
  let (a, b, c) = (window(), window(), window());
  let group = group_with(&[&b, &c]);
  let workspace =
    workspace_with(vec![a.clone().into(), group.clone().into()]);
  set_focused_descendant(&b.clone().into(), None);

  insert_window_into_group(&a, &group, &mut state).unwrap();

  assert_eq!(page_ids(&group), vec![b.id(), a.id(), c.id()]);
  assert_eq!(child_ids(&workspace), vec![group.id()]);
  assert_eq!(active_id(&group), a.id());
}

#[test]
fn removing_last_page_detaches_group() {
  let a = window();
  let x = window();
  let group = group_with(&[&a]);
  let workspace =
    workspace_with(vec![group.clone().into(), x.clone().into()]);

  detach_container(a.into()).unwrap();

  assert!(group.is_detached());
  assert_eq!(child_ids(&workspace), vec![x.id()]);
}

#[test]
fn pages_are_laid_out_below_header() {
  let a = window();
  let group = group_with(&[&a]);
  let workspace = workspace_with(vec![group.clone().into()]);

  assert_eq!(group.header_rect().unwrap(), Rect::from_xy(0, 0, 1680, 24));
  assert_eq!(a.to_rect().unwrap(), Rect::from_xy(0, 24, 1680, 976));
  assert_eq!(workspace.child_count(), 1);
}
