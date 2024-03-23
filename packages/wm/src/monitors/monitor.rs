use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonBehavior, TilingBehavior},
    Container, ContainerType,
  },
  impl_common_behavior,
};

#[derive(Clone, Debug)]
pub struct Monitor(Rc<RefCell<MonitorInner>>);

#[derive(Debug)]
struct MonitorInner {
  id: Uuid,
  parent: Option<Container>,
  children: Vec<Container>,
  native: NativeMonitor,
}

impl Monitor {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    let monitor = MonitorInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      native: native_monitor,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }
}

impl_common_behavior!(Monitor, ContainerType::Monitor);

impl TilingBehavior for Monitor {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<Container>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_tiling_children_mut(&self) -> RefMut<'_, Vec<Container>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
