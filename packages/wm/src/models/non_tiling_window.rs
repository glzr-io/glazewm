use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, DisplayState, GapsConfig, Rect, RectDelta,
  WindowDto, WindowRuleConfig, WindowState,
};
use wm_platform::NativeWindow;

use crate::{
  impl_common_getters, impl_container_debug, impl_window_getters,
  models::{
    Container, DirectionContainer, InsertionTarget, TilingContainer,
    TilingWindow, WindowContainer,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
};

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
  insertion_target: Option<InsertionTarget>,
  display_state: DisplayState,
  border_delta: RectDelta,
  has_pending_dpi_adjustment: bool,
  floating_placement: Rect,
  has_custom_floating_placement: bool,
  done_window_rules: Vec<WindowRuleConfig>,
  active_drag: Option<ActiveDrag>,
}

impl NonTilingWindow {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    id: Option<Uuid>,
    native: NativeWindow,
    state: WindowState,
    prev_state: Option<WindowState>,
    border_delta: RectDelta,
    insertion_target: Option<InsertionTarget>,
    floating_placement: Rect,
    has_custom_floating_placement: bool,
    done_window_rules: Vec<WindowRuleConfig>,
    active_drag: Option<ActiveDrag>,
  ) -> Self {
    let window = NonTilingWindowInner {
      id: id.unwrap_or_else(Uuid::new_v4),
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
      has_custom_floating_placement,
      done_window_rules,
      active_drag,
    };

    Self(Rc::new(RefCell::new(window)))
  }

  pub fn insertion_target(&self) -> Option<InsertionTarget> {
    self.0.borrow().insertion_target.clone()
  }

  pub fn set_insertion_target(
    &self,
    insertion_target: Option<InsertionTarget>,
  ) {
    self.0.borrow_mut().insertion_target = insertion_target;
  }

  pub fn to_tiling(&self, gaps_config: GapsConfig) -> TilingWindow {
    TilingWindow::new(
      Some(self.id()),
      self.native().clone(),
      Some(self.state()),
      self.border_delta(),
      self.floating_placement(),
      self.has_custom_floating_placement(),
      gaps_config,
      self.done_window_rules(),
      self.active_drag(),
    )
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;

    Ok(ContainerDto::Window(WindowDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      has_focus: self.has_focus(None),
      tiling_size: None,
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      state: self.state(),
      prev_state: self.prev_state(),
      display_state: self.display_state(),
      border_delta: self.border_delta(),
      floating_placement: self.floating_placement(),
      handle: self.native().handle,
      title: self.native().title()?,
      class_name: self.native().class_name()?,
      process_name: self.native().process_name()?,
      active_drag: self.active_drag(),
    }))
  }
}

impl_container_debug!(NonTilingWindow);
impl_common_getters!(NonTilingWindow);
impl_window_getters!(NonTilingWindow);

impl PositionGetters for NonTilingWindow {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    match self.state() {
      WindowState::Fullscreen(_) => {
        self.monitor().context("No monitor.")?.to_rect()
      }
      _ => Ok(self.floating_placement()),
    }
  }
}
