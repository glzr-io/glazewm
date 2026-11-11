use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, GroupContainerDto, GroupsConfig,
  TilingDirection,
};
use wm_platform::{Rect, TabStrip, TabStripId};

use crate::{
  impl_common_getters, impl_container_debug,
  impl_position_getters_as_resizable, impl_tiling_size_getters,
  models::{
    Container, DirectionContainer, TilingContainer, TilingWindow,
    WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters,
  },
};

/// A tiling container that holds windows as pages, of which only the
/// active page is shown.
///
/// Pages are laid out below a header strip, which is drawn by the
/// group's `TabStrip`. The first child in the child focus order is the
/// active page.
#[derive(Clone)]
pub struct GroupContainer(Rc<RefCell<GroupContainerInner>>);

struct GroupContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  gaps_config: GapsConfig,
  groups_config: GroupsConfig,
  tab_strip: Option<TabStrip>,
}

impl GroupContainer {
  /// Creates a new group container with no pages.
  pub fn new(
    gaps_config: GapsConfig,
    groups_config: GroupsConfig,
  ) -> Self {
    let group = GroupContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      gaps_config,
      groups_config,
      tab_strip: None,
    };

    Self(Rc::new(RefCell::new(group)))
  }

  /// Updates the groups config used for the header height.
  pub fn set_groups_config(&self, groups_config: GroupsConfig) {
    self.0.borrow_mut().groups_config = groups_config;
  }

  /// Gets the bounds of the header strip.
  ///
  /// The header spans the full width of the group, at its top edge.
  pub fn header_rect(&self) -> anyhow::Result<Rect> {
    let rect = self.to_rect()?;
    let header_height = self.header_height(rect.height());

    Ok(Rect::from_xy(
      rect.x(),
      rect.y(),
      rect.width(),
      header_height,
    ))
  }

  /// Gets the bounds available to pages, which is the group's bounds
  /// excluding the header.
  pub fn content_rect(&self) -> anyhow::Result<Rect> {
    let rect = self.to_rect()?;
    let header_height = self.header_height(rect.height());

    Ok(Rect::from_xy(
      rect.x(),
      rect.y() + header_height,
      rect.width(),
      (rect.height() - header_height).max(0),
    ))
  }

  /// Gets the active page, which is the first child in the child focus
  /// order.
  pub fn active_child(&self) -> Option<Container> {
    self.child_focus_order().next()
  }

  /// Gets the active page if it's a tiling window.
  pub fn active_window(&self) -> Option<TilingWindow> {
    self
      .active_child()
      .and_then(|child| child.as_tiling_window().cloned())
  }

  /// Gets the index of the active page amongst the group's pages.
  ///
  /// Returns 0 if the group has no pages.
  pub fn active_index(&self) -> usize {
    self.active_child().map_or(0, |child| child.index())
  }

  /// Gets the tab strip for the group, if one has been created.
  pub fn tab_strip(&self) -> RefMut<'_, Option<TabStrip>> {
    RefMut::map(self.0.borrow_mut(), |inner| &mut inner.tab_strip)
  }

  /// Gets the ID of the group's tab strip, if one has been created.
  pub fn tab_strip_id(&self) -> Option<TabStripId> {
    self.0.borrow().tab_strip.as_ref().map(TabStrip::id)
  }

  /// Removes and returns the group's tab strip.
  ///
  /// Dropping the returned strip destroys its window.
  pub fn take_tab_strip(&self) -> Option<TabStrip> {
    self.0.borrow_mut().tab_strip.take()
  }

  /// Returns the header height in pixels for a given group height.
  fn header_height(&self, group_height: i32) -> i32 {
    self
      .0
      .borrow()
      .groups_config
      .header_height
      .to_px(group_height, None)
      .clamp(0, group_height.max(0))
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;

    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    Ok(ContainerDto::Group(GroupContainerDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      tiling_size: self.tiling_size(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
    }))
  }
}

impl_container_debug!(GroupContainer);
impl_common_getters!(GroupContainer);
impl_tiling_size_getters!(GroupContainer);
impl_position_getters_as_resizable!(GroupContainer);

/// Gets the group that a container is a page of, if any.
pub fn enclosing_group<T: CommonGetters + ?Sized>(
  container: &T,
) -> Option<GroupContainer> {
  container
    .parent()
    .and_then(|parent| parent.as_group().cloned())
}

/// Whether a container is inside an inactive page of a group.
///
/// Pages other than a group's active page are not displayed.
pub fn is_on_inactive_page<T: CommonGetters + ?Sized>(
  container: &T,
) -> bool {
  container.self_and_ancestors().any(|ancestor| {
    ancestor.parent().is_some_and(|parent| {
      parent.is_group() && ancestor.focus_index() != 0
    })
  })
}
