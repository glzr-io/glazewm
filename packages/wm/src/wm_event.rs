use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WmEvent {
  BindingModeChanged {
    new_binding_mode: String,
  },
  FocusChanged {
    focused_container: &dyn Container,
  },
  FocusedContainerMoved {
    focused_container: &dyn Container,
  },
  MonitorAdded {
    added_monitor: &dyn Container,
  },
  MonitorRemoved {
    removed_id: Guid,
    removed_device_name: String,
  },
  NativeFocusSynced {
    focused_container: &dyn Container,
  },
  TilingDirectionChanged {
    new_tiling_direction: TilingDirection,
  },
  UserConfigReloaded,
  WindowManaged {
    managed_window: &dyn Container,
  },
  WindowUnmanaged {
    unmanaged_id: Guid,
    unmanaged_handle: WindowHandle,
  },
  WorkspaceActivated {
    activated_workspace: &dyn Container,
  },
  WorkspaceDeactivated {
    deactivated_id: Guid,
    deactivated_name: String,
  },
  WorkingAreaResized {
    affected_monitor: &dyn Container,
  },
}
