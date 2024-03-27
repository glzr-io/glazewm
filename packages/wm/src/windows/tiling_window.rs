use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::platform::NativeWindow,
  containers::{
    traits::{CommonBehavior, PositionBehavior, TilingBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_position_behavior_as_resizable,
  impl_tiling_behavior,
};

#[derive(Clone, Debug)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

#[derive(Debug)]
struct TilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: VecDeque<Container>,
  size_percent: f32,
  native: NativeWindow,
}

impl TilingWindow {
  pub fn new(native_window: NativeWindow) -> Self {
    let window = TilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      size_percent: 1.0,
      native: native_window,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(TilingWindow, ContainerType::Window);
impl_tiling_behavior!(TilingWindow);
impl_position_behavior_as_resizable!(TilingWindow);
