use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeWindow,
  containers::{traits::CommonContainer, Container, ContainerType},
  impl_common_container,
};

#[derive(Clone, Debug)]
pub struct NonTilingWindow(Rc<RefCell<NonTilingWindowInner>>);

#[derive(Debug)]
struct NonTilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  native: NativeWindow,
}

impl NonTilingWindow {
  pub fn new(native_window: NativeWindow) -> Self {
    let window = NonTilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      native: native_window,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_container!(NonTilingWindow, ContainerType::Window);
