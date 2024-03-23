use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeWindow,
  containers::{
    traits::{CommonBehavior, TilingBehavior},
    ContainerType, TilingContainer,
  },
  impl_common_behavior,
};

#[derive(Clone, Debug)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

#[derive(Debug)]
struct TilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<TilingContainer>,
  native: NativeWindow,
}

impl TilingWindow {
  pub fn new(native_window: NativeWindow) -> Self {
    let window = TilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      native: native_window,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(TilingWindow, ContainerType::Window);

impl TilingBehavior for TilingWindow {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<TilingContainer>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_tiling_children_mut(
    &self,
  ) -> RefMut<'_, Vec<TilingContainer>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
