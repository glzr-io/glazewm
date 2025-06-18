use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use ambassador::Delegate;
use enum_as_inner::EnumAsInner;
use uuid::Uuid;
use wm_common::{
  ActiveDrag, ContainerDto, Direction, DisplayState, GapsConfig, Rect,
  RectDelta, TilingDirection, WindowRuleConfig, WindowState,
};
use wm_platform::NativeWindow;

#[allow(clippy::wildcard_imports)]
use crate::{
  models::{
    Monitor, NonTilingWindow, RootContainer, SplitContainer, TilingWindow,
    Workspace,
  },
  traits::*,
  user_config::UserConfig,
};

/// A container of any type.
#[derive(Clone, Debug, EnumAsInner, Delegate, wm_macros::SubEnum)]
#[delegate(CommonGetters)]
#[delegate(PositionGetters)]
#[subenum(defaults(derives = (Clone, Debug, EnumAsInner, Delegate), delegates = (CommonGetters, PositionGetters)))]
#[subenum(
  doc = "Subset of containers that implement the following traits:"
)]
#[subenum(doc = " * `CommonGetters`")]
#[subenum(doc = " * `PositionGetters`")]
#[subenum(doc = " * `TilingSizeGetters`")]
#[subenum(TilingContainer, delegates = (TilingSizeGetters))]
#[subenum(
  doc = "Subset of containers that implement the following traits:"
)]
#[subenum(doc = " * `CommonGetters`")]
#[subenum(doc = " * `PositionGetters`")]
#[subenum(doc = " * `WindowGetters`")]
#[subenum(WindowContainer, delegates = (WindowGetters))]
#[subenum(
  doc = "  Subset of containers that implement the following traits:"
)]
#[subenum(doc = " * `CommonGetters`")]
#[subenum(doc = " * `PositionGetters`")]
#[subenum(doc = " * `TilingDirectionGetters`")]
#[subenum(DirectionContainer, delegates = (TilingDirectionGetters))]
pub enum Container {
  Root(RootContainer),
  Monitor(Monitor),
  #[subenum(DirectionContainer)]
  Workspace(Workspace),
  #[subenum(TilingContainer, DirectionContainer)]
  Split(SplitContainer),
  #[subenum(TilingContainer, WindowContainer)]
  TilingWindow(TilingWindow),
  #[subenum(WindowContainer)]
  NonTilingWindow(NonTilingWindow),
}

impl From<RootContainer> for Container {
  fn from(value: RootContainer) -> Self {
    Container::Root(value)
  }
}

impl From<Monitor> for Container {
  fn from(value: Monitor) -> Self {
    Container::Monitor(value)
  }
}

impl From<Workspace> for Container {
  fn from(value: Workspace) -> Self {
    Container::Workspace(value)
  }
}

impl From<SplitContainer> for Container {
  fn from(value: SplitContainer) -> Self {
    Container::Split(value)
  }
}

impl From<NonTilingWindow> for Container {
  fn from(value: NonTilingWindow) -> Self {
    Container::NonTilingWindow(value)
  }
}

impl From<TilingWindow> for Container {
  fn from(value: TilingWindow) -> Self {
    Container::TilingWindow(value)
  }
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

impl From<DirectionContainer> for Container {
  fn from(direction_container: DirectionContainer) -> Self {
    match direction_container {
      DirectionContainer::Split(c) => Container::Split(c),
      DirectionContainer::Workspace(c) => Container::Workspace(c),
    }
  }
}

impl PartialEq for Container {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Container {}

impl From<SplitContainer> for TilingContainer {
  fn from(value: SplitContainer) -> Self {
    TilingContainer::Split(value)
  }
}

impl From<TilingWindow> for TilingContainer {
  fn from(value: TilingWindow) -> Self {
    TilingContainer::TilingWindow(value)
  }
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

impl From<TilingWindow> for WindowContainer {
  fn from(value: TilingWindow) -> Self {
    WindowContainer::TilingWindow(value)
  }
}

impl From<NonTilingWindow> for WindowContainer {
  fn from(value: NonTilingWindow) -> Self {
    WindowContainer::NonTilingWindow(value)
  }
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
      TilingContainer::Split(_) => {
        Err("Cannot convert type to a `WindowContainer`.")
      }
    }
  }
}

impl PartialEq for WindowContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for WindowContainer {}

impl std::fmt::Display for WindowContainer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let native = self.native();
    let title = native.title().unwrap_or_default();
    let class = native.class_name().unwrap_or_default();
    let process = native.process_name().unwrap_or_default();

    // Truncate title if longer than 20 chars. Need to use `chars()`
    // instead of byte slices to handle invalid byte indices.
    let title = if title.len() > 20 {
      format!("{}...", &title.chars().take(17).collect::<String>())
    } else {
      title
    };

    write!(
      f,
      "Window(hwnd={}, process={}, class={}, title={})",
      native.handle, process, class, title,
    )
  }
}

impl From<Workspace> for DirectionContainer {
  fn from(value: Workspace) -> Self {
    DirectionContainer::Workspace(value)
  }
}

impl From<SplitContainer> for DirectionContainer {
  fn from(value: SplitContainer) -> Self {
    DirectionContainer::Split(value)
  }
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
      TilingContainer::TilingWindow(_) => {
        Err("Cannot convert type to a `DirectionContainer`.")
      }
    }
  }
}

impl PartialEq for DirectionContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for DirectionContainer {}

/// Implements the `Debug` trait for a given container struct.
///
/// Expects that the struct has a `to_dto()` method.
#[macro_export]
macro_rules! impl_container_debug {
  ($type:ty) => {
    impl std::fmt::Debug for $type {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(
          &self.to_dto().map_err(|_| std::fmt::Error),
          f,
        )
      }
    }
  };
}
