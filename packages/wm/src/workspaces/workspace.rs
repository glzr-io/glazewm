use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::RectDelta,
  containers::{
    traits::{CommonBehavior, PositionBehavior, TilingBehavior},
    Container, ContainerType, TilingContainer,
  },
  impl_common_behavior, impl_tiling_behavior,
  user_config::WorkspaceConfig,
};

#[derive(Clone, Debug)]
pub struct Workspace(Rc<RefCell<WorkspaceInner>>);

#[derive(Debug)]
struct WorkspaceInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  tiling_children: Vec<TilingContainer>,
  // TODO: Consider changing `non_tiling_children` to several fields for
  // each window type (ie. `floating_windows`, `maximized_windows`, etc.)
  non_tiling_children: Vec<Container>,
  size_percent: f32,
  config: WorkspaceConfig,
  outer_gaps: RectDelta,
}

impl Workspace {
  pub fn new(config: WorkspaceConfig, outer_gaps: RectDelta) -> Self {
    let workspace = WorkspaceInner {
      id: Uuid::new_v4(),
      parent: None,
      tiling_children: Vec::new(),
      non_tiling_children: Vec::new(),
      size_percent: 1.0,
      config,
      outer_gaps,
    };

    Self(Rc::new(RefCell::new(workspace)))
  }

  pub fn outer_gaps(&self) -> &RectDelta {
    &self.0.borrow().outer_gaps
  }
}

impl_common_behavior!(Workspace, ContainerType::Workspace);
impl_tiling_behavior!(Workspace);

impl PositionBehavior for Workspace {
  fn width(&self) -> i32 {
    self.parent().unwrap().width()
      - self.outer_gaps().left.amount
      - self.outer_gaps().right.amount
  }

  fn height(&self) -> i32 {
    self.parent().unwrap().height()
      - self.outer_gaps().top.amount
      - self.outer_gaps().bottom.amount
  }

  fn x(&self) -> i32 {
    self.parent().unwrap().x() + self.outer_gaps().left.amount
  }

  fn y(&self) -> i32 {
    self.parent().unwrap().y() + self.outer_gaps().top.amount
  }
}
