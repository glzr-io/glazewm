use enum_as_inner::EnumAsInner;
use enum_dispatch::enum_dispatch;

use crate::{
  monitors::Monitor,
  windows::{NonTilingWindow, TilingWindow},
  workspaces::Workspace,
};

use super::{traits::CommonBehavior, RootContainer, SplitContainer};

/// A reference to a container of any type.
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonBehavior, PositionBehavior)]
pub enum Container {
  Root(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  Split(SplitContainer),
  TilingWindow(TilingWindow),
  NonTilingWindow(NonTilingWindow),
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
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonBehavior, PositionBehavior, TilingBehavior)]
pub enum TilingContainer {
  Root(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  Split(SplitContainer),
  TilingWindow(TilingWindow),
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

/// A reference to a window container.
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonBehavior, PositionBehavior, WindowBehavior)]
pub enum WindowContainer {
  TilingWindow(TilingWindow),
  NonTilingWindow(NonTilingWindow),
}

impl TryFrom<Container> for WindowContainer {
  type Error = &'static str;

  fn try_from(container: Container) -> Result<Self, Self::Error> {
    match container {
      Container::TilingWindow(c) => Ok(WindowContainer::TilingWindow(c)),
      Container::NonTilingWindow(c) => {
        Ok(WindowContainer::NonTilingWindow(c))
      }
      _ => Err("Cannot convert type to a `WindowContainer`."),
    }
  }
}

impl TryFrom<TilingContainer> for WindowContainer {
  type Error = &'static str;

  fn try_from(container: TilingContainer) -> Result<Self, Self::Error> {
    match container {
      TilingContainer::TilingWindow(c) => {
        Ok(WindowContainer::TilingWindow(c))
      }
      _ => Err("Cannot convert type to a `WindowContainer`."),
    }
  }
}

impl PartialEq for WindowContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for WindowContainer {}
