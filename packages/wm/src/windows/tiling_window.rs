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
    traits::{CommonBehavior, PositionBehavior, TilingBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_position_behavior_as_resizable,
  impl_tiling_behavior, impl_window_behavior,
};

use super::{traits::WindowBehavior, WindowState};

#[derive(Clone, Debug)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

#[derive(Debug)]
struct TilingWindowInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  size_percent: f32,
  native: NativeWindow,
  state: WindowState,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
}

impl TilingWindow {
  pub fn new(
    native_window: NativeWindow,
    floating_placement: Rect,
  ) -> Self {
    let window = TilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      size_percent: 1.0,
      native: native_window,
      state: WindowState::Tiling,
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

impl_common_behavior!(TilingWindow, ContainerType::Window);
impl_tiling_behavior!(TilingWindow);
impl_position_behavior_as_resizable!(TilingWindow);
impl_window_behavior!(TilingWindow);
