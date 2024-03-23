use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::RectDelta,
  containers::{
    traits::{CommonBehavior, TilingBehavior},
    ContainerType, TilingContainer,
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
  non_tiling_children: Vec<TilingContainer>,
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
}

impl_common_behavior!(Workspace, ContainerType::Workspace);
impl_tiling_behavior!(Workspace);
