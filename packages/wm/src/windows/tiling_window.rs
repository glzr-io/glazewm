use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  fmt,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;

use crate::{
  common::{
    platform::NativeWindow, DisplayState, LengthValue, Rect, RectDelta,
    TilingDirection,
  },
  containers::{
    traits::{
      CommonGetters, PositionGetters, TilingDirectionGetters,
      TilingSizeGetters,
    },
    Container, ContainerType, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_position_getters_as_resizable,
  impl_tiling_size_getters, impl_window_getters,
};

use super::{traits::WindowGetters, WindowState};

#[derive(Clone)]
pub struct TilingWindow(Rc<RefCell<TilingWindowInner>>);

struct TilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  native: NativeWindow,
  state: WindowState,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
  inner_gap: LengthValue,
}

impl TilingWindow {
  pub fn new(
    native_window: NativeWindow,
    floating_placement: Rect,
    inner_gap: LengthValue,
  ) -> Self {
    let window = TilingWindowInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
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
      inner_gap,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn inner_gap(&self) -> LengthValue {
    self.0.borrow().inner_gap.clone()
  }

  pub fn to_dto(&self) -> anyhow::Result<TilingWindowDto> {
    Ok(TilingWindowDto {
      id: self.id(),
      parent: self.parent().map(|p| p.id()),
      tiling_size: self.tiling_size(),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      state: self.state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      has_pending_dpi_adjustment: self.has_pending_dpi_adjustment(),
      floating_placement: self.floating_placement(),
    })
  }
}

impl_common_getters!(TilingWindow, ContainerType::Window);
impl_tiling_size_getters!(TilingWindow);
impl_position_getters_as_resizable!(TilingWindow);
impl_window_getters!(TilingWindow);

impl fmt::Debug for TilingWindow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.to_dto().map_err(|_| std::fmt::Error), f)
  }
}

/// User-friendly representation of a tiling window.
///
/// Used for IPC and debug logging.
#[derive(Debug)]
pub struct TilingWindowDto {
  id: Uuid,
  parent: Option<Uuid>,
  tiling_size: f32,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  state: WindowState,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
}
