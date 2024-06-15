use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
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
    Container, ContainerDto, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_container_debug,
  impl_position_getters_as_resizable, impl_tiling_size_getters,
  impl_window_getters,
};

use super::{
  traits::WindowGetters, NonTilingWindow, WindowDto, WindowState,
};

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
  prev_state: Option<WindowState>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
  inner_gap: LengthValue,
}

impl TilingWindow {
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    prev_state: Option<WindowState>,
    border_delta: RectDelta,
    floating_placement: Rect,
    inner_gap: LengthValue,
  ) -> Self {
    let window = TilingWindowInner {
      id: id.unwrap_or_else(|| Uuid::new_v4()),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      native,
      state: WindowState::Tiling,
      prev_state,
      display_state: DisplayState::Shown,
      border_delta,
      has_pending_dpi_adjustment: false,
      floating_placement,
      inner_gap,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn to_non_tiling(
    &self,
    state: WindowState,
    insertion_target: Option<(Container, usize)>,
  ) -> NonTilingWindow {
    NonTilingWindow::new(
      Some(self.id()),
      std::mem::take(&mut self.0.borrow_mut().native),
      state,
      Some(WindowState::Tiling),
      self.border_delta(),
      insertion_target,
      self.floating_placement(),
    )
  }

  fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    Ok(ContainerDto::Window(WindowDto {
      id: self.id(),
      parent: self.parent().map(|parent| parent.id()),
      tiling_size: Some(self.tiling_size()),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      state: self.state(),
      prev_state: self.prev_state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      floating_placement: self.floating_placement(),
    }))
  }
}

impl_container_debug!(TilingWindow);
impl_common_getters!(TilingWindow);
impl_tiling_size_getters!(TilingWindow);
impl_position_getters_as_resizable!(TilingWindow);
impl_window_getters!(TilingWindow);
