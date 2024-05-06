use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  fmt,
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::{
    platform::NativeWindow, DisplayState, LengthValue, Rect, RectDelta,
  },
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, ContainerType, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_window_getters,
};

use super::{traits::WindowGetters, TilingWindow, WindowState};

#[derive(Clone)]
pub struct NonTilingWindow(Rc<RefCell<NonTilingWindowInner>>);

struct NonTilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  native: NativeWindow,
  state: WindowState,
  prev_state: Option<WindowState>,
  insertion_target: Option<(Container, usize)>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
}

impl NonTilingWindow {
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    state: WindowState,
    prev_state: Option<WindowState>,
    insertion_target: Option<(Container, usize)>,
    floating_placement: Rect,
  ) -> Self {
    let window = NonTilingWindowInner {
      id: id.unwrap_or_else(|| Uuid::new_v4()),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      native,
      state,
      prev_state,
      insertion_target,
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

  pub fn insertion_target(&self) -> Option<(Container, usize)> {
    self.0.borrow().insertion_target.clone()
  }

  pub fn set_state(&self, state: WindowState) {
    self.0.borrow_mut().state = state;
  }

  pub fn to_tiling(&self, inner_gap: LengthValue) -> TilingWindow {
    TilingWindow::new(
      Some(self.id()),
      self.native(),
      Some(self.state()),
      self.floating_placement(),
      inner_gap,
    )
  }

  pub fn to_dto(&self) -> anyhow::Result<NonTilingWindowDto> {
    Ok(NonTilingWindowDto {
      id: self.id(),
      parent: self.parent().map(|p| p.id()),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      state: self.state(),
      prev_state: self.prev_state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      has_pending_dpi_adjustment: self.has_pending_dpi_adjustment(),
      floating_placement: self.floating_placement(),
    })
  }
}

impl_common_getters!(NonTilingWindow, ContainerType::Window);
impl_window_getters!(NonTilingWindow);

impl PositionGetters for NonTilingWindow {
  fn width(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().floating_placement.width())
  }

  fn height(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().floating_placement.height())
  }

  fn x(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().floating_placement.x())
  }

  fn y(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().floating_placement.y())
  }
}

impl fmt::Debug for NonTilingWindow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.to_dto().map_err(|_| std::fmt::Error), f)
  }
}

/// User-friendly representation of a non-tiling window.
///
/// Used for IPC and debug logging.
#[derive(Debug)]
pub struct NonTilingWindowDto {
  id: Uuid,
  parent: Option<Uuid>,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  state: WindowState,
  prev_state: Option<WindowState>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
}
