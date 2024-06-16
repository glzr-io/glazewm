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
  },
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, ContainerDto, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_container_debug, impl_window_getters,
};

use super::{traits::WindowGetters, TilingWindow, WindowDto, WindowState};

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
    border_delta: RectDelta,
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
      border_delta,
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
    let native = std::mem::take(&mut self.0.borrow_mut().native);

    TilingWindow::new(
      Some(self.id()),
      native,
      Some(self.state()),
      self.border_delta(),
      self.floating_placement(),
      inner_gap,
    )
  }

  fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    Ok(ContainerDto::Window(WindowDto {
      id: self.id(),
      parent: self.parent().map(|parent| parent.id()),
      tiling_size: None,
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

impl_container_debug!(NonTilingWindow);
impl_common_getters!(NonTilingWindow);
impl_window_getters!(NonTilingWindow);

impl PositionGetters for NonTilingWindow {
  fn width(&self) -> anyhow::Result<i32> {
    // TODO: Simplify with a `borrow_monitor_rect` fn.
    let width = match self.state() {
      WindowState::Fullscreen(_) => self
        .monitor()
        .context("No monitor.")?
        .native()
        .rect()?
        .width(),
      _ => self.floating_placement().width(),
    };

    Ok(width)
  }

  fn height(&self) -> anyhow::Result<i32> {
    // TODO: Simplify with a `borrow_monitor_rect` fn.
    let height = match self.state() {
      WindowState::Fullscreen(_) => self
        .monitor()
        .context("No monitor.")?
        .native()
        .rect()?
        .height(),
      _ => self.floating_placement().height(),
    };

    Ok(height)
  }

  fn x(&self) -> anyhow::Result<i32> {
    // TODO: Simplify with a `borrow_monitor_rect` fn.
    let x = match self.state() {
      WindowState::Fullscreen(_) => {
        self.monitor().context("No monitor.")?.native().rect()?.x()
      }
      _ => self.floating_placement().x(),
    };

    Ok(x)
  }

  fn y(&self) -> anyhow::Result<i32> {
    // TODO: Simplify with a `borrow_monitor_rect` fn.
    let y = match self.state() {
      WindowState::Fullscreen(_) => {
        self.monitor().context("No monitor.")?.native().rect()?.y()
      }
      _ => self.floating_placement().y(),
    };

    Ok(y)
  }
}
