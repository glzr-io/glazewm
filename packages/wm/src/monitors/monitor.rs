use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonContainer, TilingContainer},
    ContainerRef, ContainerType,
  },
};

#[derive(Clone, Debug)]
pub struct MonitorRef(Rc<RefCell<Monitor>>);

#[derive(Debug)]
pub struct Monitor {
  id: Uuid,
  pub parent: Option<ContainerRef>,
  pub children: Vec<ContainerRef>,
  pub device_name: String,
  pub width: i32,
  pub height: i32,
  pub x: i32,
  pub y: i32,
}

impl MonitorRef {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    let monitor = Monitor {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      device_name: native_monitor.device_name,
      width: native_monitor.width,
      height: native_monitor.height,
      x: native_monitor.x,
      y: native_monitor.y,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }
}

impl CommonContainer for MonitorRef {
  fn id(&self) -> Uuid {
    self.0.borrow().id
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::Monitor
  }
}

impl TilingContainer for MonitorRef {
  fn borrow_parent(&self) -> Ref<'_, Option<ContainerRef>> {
    Ref::map(self.0.borrow(), |c| &c.parent)
  }

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<ContainerRef>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.parent)
  }

  fn borrow_children(&self) -> Ref<'_, Vec<ContainerRef>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_children_mut(&self) -> RefMut<'_, Vec<ContainerRef>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
