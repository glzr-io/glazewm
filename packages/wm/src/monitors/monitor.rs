use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonBehavior, PositionBehavior, TilingBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_tiling_behavior,
};

#[derive(Clone, Debug)]
pub struct Monitor(Rc<RefCell<MonitorInner>>);

#[derive(Debug)]
struct MonitorInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<Container>,
  size_percent: f32,
  native: NativeMonitor,
}

impl Monitor {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    let monitor = MonitorInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      size_percent: 1.0,
      native: native_monitor,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }

  pub fn native(&self) -> NativeMonitor {
    todo!()
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
