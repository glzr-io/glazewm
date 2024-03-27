use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::{platform::NativeWindow, Rect},
  containers::{
    traits::{CommonBehavior, PositionBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior,
};

#[derive(Clone, Debug)]
pub struct NonTilingWindow(Rc<RefCell<NonTilingWindowInner>>);

#[derive(Debug)]
struct NonTilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<Container>,
  native: NativeWindow,
  position: Rect,
}

impl NonTilingWindow {
  pub fn new(native_window: NativeWindow) -> Self {
    let window = NonTilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      native: native_window,
      position: Rect::from_xy(0, 0, 0, 0),
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(NonTilingWindow, ContainerType::Window);

impl PositionBehavior for NonTilingWindow {
  fn width(&self) -> i32 {
    self.0.borrow().position.width()
  }

  fn height(&self) -> i32 {
    self.0.borrow().position.height()
  }

  fn x(&self) -> i32 {
    self.0.borrow().position.x()
  }

  fn y(&self) -> i32 {
    self.0.borrow().position.y()
  }
}
