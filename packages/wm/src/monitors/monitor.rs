use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonBehavior, PositionBehavior, TilingBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_tiling_behavior,
  workspaces::Workspace,
};

#[derive(Clone, Debug)]
pub struct Monitor(Rc<RefCell<MonitorInner>>);

#[derive(Debug)]
struct MonitorInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  size_percent: f32,
  native: NativeMonitor,
}

impl Monitor {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    let monitor = MonitorInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      size_percent: 1.0,
      native: native_monitor,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }

  pub fn native(&self) -> NativeMonitor {
    self.0.borrow().native.clone()
  }

  pub fn name(&self) -> String {
    self.native().device_name
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    todo!()
  }

  pub fn displayed_workspace(&self) -> Option<Workspace> {
    self
      .borrow_children()
      .front()
      .and_then(|c| c.as_workspace())
      .cloned()
  }

  /// Whether there is a difference in DPI between this monitor and the
  /// parent monitor of another container.
  pub fn has_dpi_difference(
    &self,
    other: &Container,
  ) -> anyhow::Result<bool> {
    let other_monitor =
      other.parent_monitor().context("No parent monitor.")?;

    Ok(self.native().dpi != other_monitor.native().dpi)
  }
}

impl_common_behavior!(Monitor, ContainerType::Monitor);
impl_tiling_behavior!(Monitor);

impl PositionBehavior for Monitor {
  fn width(&self) -> i32 {
    self.0.borrow().native.width
  }

  fn height(&self) -> i32 {
    self.0.borrow().native.height
  }

  fn x(&self) -> i32 {
    self.0.borrow().native.x
  }

  fn y(&self) -> i32 {
    self.0.borrow().native.y
  }
}
