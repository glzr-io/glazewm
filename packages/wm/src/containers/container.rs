use enum_dispatch::enum_dispatch;

use crate::{
  monitors::Monitor,
  windows::{NonTilingWindow, TilingWindow},
  workspaces::Workspace,
};

use super::{traits::CommonBehavior, RootContainer, SplitContainer};

/// A reference to a container of any type.
#[derive(Clone, Debug)]
#[enum_dispatch(CommonBehavior, PositionBehavior)]
pub enum Container {
  Root(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  Split(SplitContainer),
  TilingWindow(TilingWindow),
  NonTilingWindow(NonTilingWindow),
}

impl Container {
  pub fn as_monitor(&self) -> Option<Monitor> {
    match self {
      Container::Monitor(c) => Some(c.clone()),
      _ => None,
    }
  }

  pub fn as_workspace(&self) -> Option<Workspace> {
    match self {
      Container::Workspace(c) => Some(c.clone()),
      _ => None,
    }
  }
}

impl From<TilingContainer> for Container {
  fn from(tiling_container: TilingContainer) -> Self {
    match tiling_container {
      TilingContainer::Root(c) => Container::Root(c),
      TilingContainer::Monitor(c) => Container::Monitor(c),
      TilingContainer::Workspace(c) => Container::Workspace(c),
      TilingContainer::Split(c) => Container::Split(c),
      TilingContainer::TilingWindow(c) => Container::TilingWindow(c),
    }
  }
}

impl PartialEq for Container {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Container {}

/// A reference to a tiling container.
#[derive(Clone, Debug)]
#[enum_dispatch(CommonBehavior, PositionBehavior, TilingBehavior)]
pub enum TilingContainer {
  Root(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  Split(SplitContainer),
  TilingWindow(TilingWindow),
}

impl TilingContainer {
  pub fn as_monitor(&self) -> Option<Monitor> {
    match self {
      TilingContainer::Monitor(c) => Some(c.clone()),
      _ => None,
    }
  }

  pub fn as_workspace(&self) -> Option<Workspace> {
    match self {
      TilingContainer::Workspace(c) => Some(c.clone()),
      _ => None,
    }
  }
}

impl TryFrom<Container> for TilingContainer {
  type Error = &'static str;

  fn try_from(container: Container) -> Result<Self, Self::Error> {
    match container {
      Container::Root(c) => Ok(TilingContainer::Root(c)),
      Container::Monitor(c) => Ok(TilingContainer::Monitor(c)),
      Container::Workspace(c) => Ok(TilingContainer::Workspace(c)),
      Container::Split(c) => Ok(TilingContainer::Split(c)),
      Container::TilingWindow(c) => Ok(TilingContainer::TilingWindow(c)),
      Container::NonTilingWindow(_) => {
        Err("Cannot convert `NonTilingWindow` to `TilingContainer`.")
      }
    }
  }
}

impl PartialEq for TilingContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for TilingContainer {}
