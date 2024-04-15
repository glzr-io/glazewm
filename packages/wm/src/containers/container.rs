use enum_as_inner::EnumAsInner;
use enum_dispatch::enum_dispatch;

use crate::{
  monitors::Monitor,
  windows::{NonTilingWindow, TilingWindow},
  workspaces::Workspace,
};

use super::{traits::CommonGetters, RootContainer, SplitContainer};

/// A container of any type.
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonGetters, PositionGetters)]
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
      TilingContainer::Split(c) => Container::Split(c),
      TilingContainer::TilingWindow(c) => Container::TilingWindow(c),
    }
  }
}

impl From<WindowContainer> for Container {
  fn from(window_container: WindowContainer) -> Self {
    match window_container {
      WindowContainer::NonTilingWindow(c) => Container::NonTilingWindow(c),
      WindowContainer::TilingWindow(c) => Container::TilingWindow(c),
    }
  }
}

impl PartialEq for Container {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Container {}

/// Subset of containers that implement the following traits:
///  * `CommonGetters`
///  * `PositionGetters`
///  * `TilingSizeGetters`
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonGetters, PositionGetters, TilingSizeGetters)]
pub enum TilingContainer {
  Split(SplitContainer),
  TilingWindow(TilingWindow),
}

impl TryFrom<Container> for TilingContainer {
  type Error = &'static str;

  fn try_from(container: Container) -> Result<Self, Self::Error> {
    match container {
      Container::Split(c) => Ok(TilingContainer::Split(c)),
      Container::TilingWindow(c) => Ok(TilingContainer::TilingWindow(c)),
      _ => Err("Cannot convert type to `TilingContainer`."),
    }
  }
}

impl PartialEq for TilingContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for TilingContainer {}

/// Subset of containers that implement the following traits:
///  * `CommonGetters`
///  * `PositionGetters`
///  * `WindowGetters`
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonGetters, PositionGetters, WindowGetters)]
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

/// Subset of containers that implement the following traits:
///  * `CommonGetters`
///  * `PositionGetters`
///  * `TilingDirectionGetters`
#[derive(Clone, Debug, EnumAsInner)]
#[enum_dispatch(CommonGetters, PositionGetters, TilingDirectionGetters)]
pub enum DirectionContainer {
  Workspace(Workspace),
  Split(SplitContainer),
}

impl TryFrom<Container> for DirectionContainer {
  type Error = &'static str;

  fn try_from(container: Container) -> Result<Self, Self::Error> {
    match container {
      Container::Workspace(c) => Ok(DirectionContainer::Workspace(c)),
      Container::Split(c) => Ok(DirectionContainer::Split(c)),
      _ => Err("Cannot convert type to a `DirectionContainer`."),
    }
  }
}

impl TryFrom<TilingContainer> for DirectionContainer {
  type Error = &'static str;

  fn try_from(container: TilingContainer) -> Result<Self, Self::Error> {
    match container {
      TilingContainer::Split(c) => Ok(DirectionContainer::Split(c)),
      _ => Err("Cannot convert type to a `DirectionContainer`."),
    }
  }
}

impl PartialEq for DirectionContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for DirectionContainer {}
