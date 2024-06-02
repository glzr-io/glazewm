use serde::Serialize;
use uuid::Uuid;

use crate::{
  common::TilingDirection,
  containers::{Container, WindowContainer},
  monitors::Monitor,
  user_config::{BindingModeConfig, ParsedConfig},
  workspaces::Workspace,
};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum WmEvent {
  BindingModesChanged {
    active_binding_modes: Vec<BindingModeConfig>,
  },
  FocusChanged {
    focused_container: Container,
  },
  FocusedContainerMoved {
    focused_container: Container,
  },
  MonitorAdded {
    added_monitor: Monitor,
  },
  MonitorUpdated {
    updated_monitor: Monitor,
  },
  MonitorRemoved {
    removed_id: Uuid,
    removed_device_name: String,
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
    managed_window: WindowContainer,
  },
  WindowUnmanaged {
    unmanaged_id: Uuid,
    unmanaged_handle: isize,
  },
  WorkspaceActivated {
    activated_workspace: Workspace,
  },
  WorkspaceDeactivated {
    deactivated_id: Uuid,
    deactivated_name: String,
  },
  WorkspaceMoved {
    workspace: Workspace,
    new_monitor: Monitor,
  },
}
