use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, Rect, TilingDirection, WorkspaceConfig,
  WorkspaceDto, WorkspaceWindowDto,
};

use crate::{
  impl_common_getters, impl_container_debug,
  impl_tiling_direction_getters,
  models::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, WindowGetters,
  },
};

#[derive(Clone)]
pub struct Workspace(Rc<RefCell<WorkspaceInner>>);

#[derive(Debug)]
struct WorkspaceInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  config: WorkspaceConfig,
  gaps_config: GapsConfig,
  tiling_direction: TilingDirection,
  window_icons_enabled: bool,
}

impl Workspace {
  pub fn new(
    config: WorkspaceConfig,
    gaps_config: GapsConfig,
    tiling_direction: TilingDirection,
    window_icons_enabled: bool,
  ) -> Self {
    let workspace = WorkspaceInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      config,
      gaps_config,
      tiling_direction,
      window_icons_enabled,
    };

    Self(Rc::new(RefCell::new(workspace)))
  }

  /// Underlying config for the workspace.
  pub fn config(&self) -> WorkspaceConfig {
    self.0.borrow().config.clone()
  }

  /// Update the underlying config for the workspace.
  pub fn set_config(&self, config: WorkspaceConfig) {
    self.0.borrow_mut().config = config;
  }

  /// Whether the workspace is currently displayed by the parent monitor.
  pub fn is_displayed(&self) -> bool {
    self
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .is_some_and(|workspace| workspace.id() == self.id())
  }

  pub fn set_gaps_config(&self, gaps_config: GapsConfig) {
    self.0.borrow_mut().gaps_config = gaps_config;
  }

  pub fn set_window_icons_enabled(&self, enabled: bool) {
    self.0.borrow_mut().window_icons_enabled = enabled;
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let include_window_icons = self.0.borrow().window_icons_enabled;

    let rect = self.to_rect()?;
    let config = self.config();

    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    // Collect all windows in this workspace if window_icons are enabled.
    let windows = if include_window_icons {
      self
        .descendants()
        .filter_map(|container| container.as_window_container().ok())
        .filter_map(|window| {
          let native = window.native();
          // Extract process name, title, and icon. If process_name or
          // title fails, skip this window by returning None.
          match (native.process_name(), native.title()) {
            (Ok(process_name), Ok(title)) => {
              let icon = native.icon_as_data_url();
              Some(WorkspaceWindowDto {
                process_name,
                title,
                icon,
              })
            }
            _ => None,
          }
        })
        .collect()
    } else {
      vec![]
    };

    Ok(ContainerDto::Workspace(WorkspaceDto {
      id: self.id(),
      name: config.name,
      display_name: config.display_name,
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      is_displayed: self.is_displayed(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      tiling_direction: self.tiling_direction(),
      windows,
    }))
  }
}

impl_container_debug!(Workspace);
impl_common_getters!(Workspace);
impl_tiling_direction_getters!(Workspace);

impl PositionGetters for Workspace {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    let monitor =
      self.monitor().context("Workspace has no parent monitor.")?;

    let gaps_config = &self.0.borrow().gaps_config;
    let scale_factor = match &gaps_config.scale_with_dpi {
      true => monitor.native().scale_factor()?,
      false => 1.,
    };

    // Get delta between monitor bounds and its working area.
    let working_delta = monitor
      .native()
      .working_rect()
      .context("Failed to get working area of parent monitor.")?
      .delta(&monitor.to_rect()?);

    let is_single_window = self.tiling_children().nth(1).is_none();

    let gaps = if is_single_window {
      gaps_config
        .single_window_outer_gap
        .as_ref()
        .unwrap_or(&gaps_config.outer_gap)
    } else {
      &gaps_config.outer_gap
    };

    Ok(
      monitor
        .to_rect()?
        // Scale the gaps if `scale_with_dpi` is enabled.
        .apply_inverse_delta(gaps, Some(scale_factor))
        .apply_delta(&working_delta, None),
    )
  }
}

impl std::fmt::Display for Workspace {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Workspace(name={}, tiling_direction={:?})",
      self.config().name,
      self.tiling_direction(),
    )
  }
}
