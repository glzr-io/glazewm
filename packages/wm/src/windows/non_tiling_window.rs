use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::{
    platform::NativeWindow, DisplayState, LengthValue, Rect, RectDelta,
  },
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
  child_focus_order: VecDeque<Uuid>,
  native: NativeWindow,
  state: WindowState,
  prev_state: Option<WindowState>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
}

impl NonTilingWindow {
  pub fn new(
    native_window: NativeWindow,
    state: WindowState,
    prev_state: Option<WindowState>,
    floating_placement: Rect,
  ) -> Self {
    let window = NonTilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      native: native_window,
      state,
      prev_state,
      display_state: DisplayState::Shown,
      border_delta: RectDelta::new(
        LengthValue::new_px(0.),
        LengthValue::new_px(0.),
        LengthValue::new_px(0.),
        LengthValue::new_px(0.),
      ),
      has_pending_dpi_adjustment: false,
      floating_placement,
    };

    Self(Rc::new(RefCell::new(window)))
  }
}

impl_common_behavior!(NonTilingWindow, ContainerType::Window);
impl_window_behavior!(NonTilingWindow);

impl PositionBehavior for NonTilingWindow {
  fn width(&self) -> i32 {
    self.0.borrow().floating_placement.width()
  }

  fn height(&self) -> i32 {
    self.0.borrow().floating_placement.height()
  }

  fn x(&self) -> i32 {
    self.0.borrow().floating_placement.x()
  }

  fn y(&self) -> i32 {
    self.0.borrow().floating_placement.y()
  }
}
