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
  impl_common_behavior, impl_tiling_behavior,
};

#[derive(Clone, Debug)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

#[derive(Debug)]
struct TilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<TilingContainer>,
  size_percent: f32,
  native: NativeWindow,
}

impl TilingWindow {
  pub fn new(native_window: NativeWindow) -> Self {
    let window = TilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      size_percent: 1.0,
      native: native_window,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(TilingWindow, ContainerType::Window);
impl_tiling_behavior!(TilingWindow);
