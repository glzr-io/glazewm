use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::RectDelta,
  containers::{
    traits::{CommonContainer, TilingContainer},
    Container, ContainerType,
  },
  impl_common_container,
  user_config::WorkspaceConfig,
};

#[derive(Clone, Debug)]
pub struct Workspace(Rc<RefCell<WorkspaceInner>>);

#[derive(Debug)]
struct WorkspaceInner {
  id: Uuid,
  parent: Option<Container>,
  tiling_children: Vec<Container>,
  non_tiling_children: Vec<Container>,
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
      config,
      outer_gaps,
    };

    Self(Rc::new(RefCell::new(workspace)))
  }
}

impl_common_container!(Workspace, ContainerType::Workspace);

impl TilingContainer for Workspace {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<Container>> {
    Ref::map(self.0.borrow(), |c| &c.tiling_children)
  }

  fn borrow_tiling_children_mut(&self) -> RefMut<'_, Vec<Container>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.tiling_children)
  }
}
