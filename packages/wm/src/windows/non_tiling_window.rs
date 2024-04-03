use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::{platform::NativeWindow, DisplayState, Rect},
  containers::{
    traits::{CommonBehavior, PositionBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_window_behavior,
};

use super::{traits::WindowBehavior, WindowState};

#[derive(Clone, Debug)]
pub struct NonTilingWindow(Rc<RefCell<NonTilingWindowInner>>);

#[derive(Debug)]
struct NonTilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: VecDeque<Container>,
  native: NativeWindow,
  position: Rect,
  state: WindowState,
  display_state: DisplayState,
  has_pending_dpi_adjustment: bool,
}

impl NonTilingWindow {
  pub fn new(native_window: NativeWindow, state: WindowState) -> Self {
    let window = NonTilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      native: native_window,
      position: Rect::from_xy(0, 0, 0, 0),
      state,
      display_state: DisplayState::Shown,
      has_pending_dpi_adjustment: false,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(NonTilingWindow, ContainerType::Window);
impl_window_behavior!(NonTilingWindow);

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
