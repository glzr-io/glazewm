use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  common::TilingDirection,
  containers::ContainerDto,
  user_config::{BindingModeConfig, ParsedConfig},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "eventType", rename_all = "snake_case")]
pub enum WmEvent {
  ApplicationExiting,
  BindingModesChanged {
    active_binding_modes: Vec<BindingModeConfig>,
  },
  FocusChanged {
    focused_container: ContainerDto,
  },
  FocusedContainerMoved {
    focused_container: ContainerDto,
  },
  MonitorAdded {
    added_monitor: ContainerDto,
  },
  MonitorRemoved {
    removed_id: Uuid,
    removed_device_name: String,
  },
  MonitorUpdated {
    updated_monitor: ContainerDto,
  },
  TilingDirectionChanged {
    modified_id: Uuid,
    new_tiling_direction: TilingDirection,
  },
  UserConfigChanged {
    config_path: String,
    config_string: String,
    parsed_config: ParsedConfig,
  },
  WindowManaged {
    managed_window: ContainerDto,
  },
  WindowUnmanaged {
    unmanaged_id: Uuid,
    unmanaged_handle: isize,
  },
  WorkspaceActivated {
    activated_workspace: ContainerDto,
  },
  WorkspaceDeactivated {
    deactivated_id: Uuid,
    deactivated_name: String,
  },
  WorkspaceMoved {
    moved_workspace: ContainerDto,
    new_monitor: ContainerDto,
  },
}
