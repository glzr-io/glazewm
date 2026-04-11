use wm_common::{FloatingStateConfig, TilingDirection, WindowState};

use crate::{
  models::{WindowContainer, *},
  traits::*,
  wm_state::WmState,
};

impl WmState {
  fn find_window_by_title(&self, title: &str) -> Option<WindowContainer> {
    self
      .windows()
      .iter()
      .find(|w| w.native_properties().title == title)
      .cloned()
  }
}

/// Tests that a `SplitContainer` with three children distributes tiling
/// sizes equally (approximately 1/3 each).
#[test]
fn test_split_container_equal_tiling_sizes() {
  let split = SplitContainer::mock()
    .tiling_containers(vec![
      TilingWindow::mock().call().into(),
      TilingWindow::mock().call().into(),
      TilingWindow::mock().call().into(),
    ])
    .call();

  let children: Vec<_> = split.tiling_children().collect();
  assert_eq!(children.len(), 3);

  let expected_size = 1.0 / 3.0;
  assert!((children[0].tiling_size() - expected_size).abs() < 0.01);
  assert!((children[1].tiling_size() - expected_size).abs() < 0.01);
  assert!((children[2].tiling_size() - expected_size).abs() < 0.01);
}

/// Tests that a simple `WmState` with one monitor correctly tracks the
/// monitor and that the monitor is properly linked to the root container.
#[test]
fn test_simple_wm_state() {
  let state = WmState::mock()
    .monitors(vec![Monitor::mock().call()])
    .call();

  let monitors = state.monitors();
  assert_eq!(monitors.len(), 1);

  let monitor = &monitors[0];
  let parent = monitor.parent().expect("monitor should have parent");
  assert!(
    parent.as_root().is_some(),
    "monitor's parent should be root container"
  );
}

/// Generates windows nested inside horizontal and vertical split
/// containers.
///
/// Structure:
/// ```text
/// Workspace
/// └── SplitContainer (Horizontal)
///     ├── SplitContainer (Vertical)
///     │   ├── SplitContainer (Horizontal)
///     │   │   ├── window1
///     │   │   └── window2
///     │   └── window3
///     └── SplitContainer (Vertical)
///         ├── window4
///         └── window5
/// ```
fn state_with_h_v_h_splits() -> WmState {
  WmState::mock()
    .monitors(vec![Monitor::mock()
      .workspaces(vec![Workspace::mock()
        .tiling_containers(vec![SplitContainer::mock()
          .tiling_direction(TilingDirection::Horizontal)
          .tiling_containers(vec![
            SplitContainer::mock()
              .tiling_direction(TilingDirection::Vertical)
              .tiling_containers(vec![
                SplitContainer::mock()
                  .tiling_direction(TilingDirection::Horizontal)
                  .tiling_containers(vec![
                    TilingWindow::mock()
                      .title("window1".into())
                      .call()
                      .into(),
                    TilingWindow::mock()
                      .title("window2".into())
                      .call()
                      .into(),
                  ])
                  .call()
                  .into(),
                TilingWindow::mock().title("window3".into()).call().into(),
              ])
              .call()
              .into(),
            SplitContainer::mock()
              .tiling_direction(TilingDirection::Vertical)
              .tiling_containers(vec![
                TilingWindow::mock().title("window4".into()).call().into(),
                TilingWindow::mock().title("window5".into()).call().into(),
              ])
              .call()
              .into(),
          ])
          .call()
          .into()])
        .call()])
      .call()])
    .call()
}

/// Tests that windows nested inside horizontal and vertical split
/// containers can be found by title via `state.windows()`.
#[test]
fn test_nested_splits() {
  let state = state_with_h_v_h_splits();

  let windows = state.windows();
  assert_eq!(windows.len(), 5);

  assert!(
    state.find_window_by_title("window1").is_some(),
    "window1 should exist"
  );
  assert!(
    state.find_window_by_title("window2").is_some(),
    "window2 should exist"
  );
  assert!(
    state.find_window_by_title("window3").is_some(),
    "window3 should exist"
  );
  assert!(
    state.find_window_by_title("window4").is_some(),
    "window4 should exist"
  );
  assert!(
    state.find_window_by_title("window5").is_some(),
    "window5 should exist"
  );
}

/// Tests that a workspace with both tiling and floating windows can find
/// both types via `state.windows()`.
#[test]
fn test_mixed_window_types() {
  let state = WmState::mock()
    .monitors(vec![Monitor::mock()
      .workspaces(vec![Workspace::mock()
        .tiling_containers(vec![TilingWindow::mock()
          .title("tiling-window".into())
          .call()
          .into()])
        .non_tiling_windows(vec![NonTilingWindow::mock()
          .title("floating-window".into())
          .state(WindowState::Floating(FloatingStateConfig::default()))
          .call()])
        .call()])
      .call()])
    .call();

  let windows = state.windows();
  assert_eq!(windows.len(), 2);

  let tiling = state.find_window_by_title("tiling-window");
  let floating = state.find_window_by_title("floating-window");

  assert!(tiling.is_some(), "tiling window should be findable");
  assert!(floating.is_some(), "floating window should be findable");

  let tiling = tiling.unwrap();
  let floating = floating.unwrap();

  assert!(
    matches!(tiling.state(), WindowState::Tiling),
    "tiling window should have Tiling state"
  );
  assert!(
    matches!(floating.state(), WindowState::Floating(_)),
    "floating window should have Floating state"
  );
}

/// Tests that windows on multiple monitors are findable and correctly
/// attached to their respective monitors.
#[test]
fn test_multiple_monitors() {
  let state = WmState::mock()
    .monitors(vec![
      Monitor::mock()
        .device_name("monitor-1".into())
        .workspaces(vec![Workspace::mock()
          .name("workspace-1".into())
          .tiling_containers(vec![
            TilingWindow::mock()
              .title("m1-window-1".into())
              .call()
              .into(),
            TilingWindow::mock()
              .title("m1-window-2".into())
              .call()
              .into(),
          ])
          .call()])
        .call(),
      Monitor::mock()
        .device_name("monitor-2".into())
        .workspaces(vec![Workspace::mock()
          .name("workspace-2".into())
          .tiling_containers(vec![TilingWindow::mock()
            .title("m2-window-1".into())
            .call()
            .into()])
          .call()])
        .call(),
    ])
    .call();

  let windows = state.windows();
  assert_eq!(windows.len(), 3);

  let monitors = state.monitors();
  assert_eq!(monitors.len(), 2);

  let m1_w1 = state
    .find_window_by_title("m1-window-1")
    .expect("m1_w1 should exist");
  let m1_w2 = state
    .find_window_by_title("m1-window-2")
    .expect("m1_w2 should exist");
  let m2_w1 = state
    .find_window_by_title("m2-window-1")
    .expect("m2_w1 should exist");

  let monitor_1 = &monitors[0];
  let monitor_2 = &monitors[1];

  let workspace_1 = monitor_1
    .workspaces()
    .into_iter()
    .next()
    .expect("monitor 1 should have workspace");
  let workspace_2 = monitor_2
    .workspaces()
    .into_iter()
    .next()
    .expect("monitor 2 should have workspace");

  assert_eq!(
    workspace_1.config().name,
    "workspace-1",
    "monitor 1 should have workspace-1"
  );
  assert_eq!(
    workspace_2.config().name,
    "workspace-2",
    "monitor 2 should have workspace-2"
  );

  assert_eq!(
    m1_w1
      .monitor()
      .expect("window should have monitor")
      .native_properties()
      .device_name,
    monitor_1.native_properties().device_name,
    "m1-window-1 should be on monitor-1"
  );
  assert_eq!(
    m1_w2
      .monitor()
      .expect("window should have monitor")
      .native_properties()
      .device_name,
    monitor_1.native_properties().device_name,
    "m1-window-2 should be on monitor-1"
  );
  assert_eq!(
    m2_w1
      .monitor()
      .expect("window should have monitor")
      .native_properties()
      .device_name,
    monitor_2.native_properties().device_name,
    "m2-window-1 should be on monitor-2"
  );
}
