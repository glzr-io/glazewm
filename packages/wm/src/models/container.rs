use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use ambassador::Delegate;
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
///
/// Uses:
///
///  * [`wm_macros::SubEnum`] to define subtypes of containers.
///  * [`wm_macros::EnumFromInner`] to define conversions between the enum
///    and wrapped types.
///  * [`ambassador::Delegate`] to delegate common getters to the contained
///    types. E.g. implements [`CommonGetters`] for [Container] by
///    forwarding the call to the item contained in the enum variant.
///
/// # Example
/// Conversion between the different container types:
/// ```
/// fn example(split: SplitContainer) {
///   // Convert a `SplitContainer` into a `Container`
///   let container: Container = split.into(); // Will be a `Container::Split`
///
///   // Could also have gone straight to a [TilingContainer] from SplitContainer
///   // let tiling: TilingContainer = split.into(); // Will be a `TilingContainer::Split`
///
///   // Try to convert a [Container] into a sub container type ([TilingContainer] in this case).
///   let tiling: TilingContainer = container.try_into().unwrap(); // Will be a `TilingContainer::Split`
///   tiling.tiling_size(); // Can use methods from the `TilingSizeGetters` trait.
///
///   // Try to convert a one sub container type into another. ([TilingContainer] to [DirectionContainer] in this case).
///   let direction: DirectionContainer = tiling.try_into().unwrap(); // Will be a `DirectionContainer::Split`
///   direction.tiling_direction(); // Can use methods from the `TilingDirectionGetters` trait.
///
///   // Covert a sub container back into a [Container]
///   let container: Container = direction.into(); // Will be a `Container::Split`
/// }
/// ```
#[derive(
  Clone, Debug, wm_macros::EnumFromInner, Delegate, wm_macros::SubEnum,
)]
#[delegate(CommonGetters)]
#[delegate(PositionGetters)]
#[subenum(defaults, {
  /// Subenum of [Container]
  #[derive(Clone, Debug, Delegate, wm_macros::EnumFromInner)]
  #[delegate(CommonGetters)]
  #[delegate(PositionGetters)]
})]
#[subenum(TilingContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `TilingSizeGetters`
  #[delegate(TilingSizeGetters)]
})]
#[subenum(WindowContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `WindowGetters`
  #[delegate(WindowGetters)]
})]
#[subenum(DirectionContainer, {
  /// Subset of containers that implement the following traits:
  /// * `CommonGetters`
  /// * `PositionGetters`
  /// * `DirectionGetters`
  #[delegate(TilingDirectionGetters)]
})]
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

impl PartialEq for Container {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for Container {}

impl PartialEq for TilingContainer {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
  }
}

impl Eq for TilingContainer {}

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
