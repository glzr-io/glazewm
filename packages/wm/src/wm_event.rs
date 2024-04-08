use uuid::Uuid;

use crate::{
  common::{platform::WindowHandle, TilingDirection},
  containers::{Container, WindowContainer},
  monitors::Monitor,
  workspaces::Workspace,
};

#[derive(Debug)]
pub enum WmEvent {
  BindingModeChanged {
    new_binding_mode: String,
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
  MonitorRemoved {
    removed_id: Uuid,
    removed_device_name: String,
  },
  NativeFocusSynced {
    focused_container: Container,
  },
  TilingDirectionChanged {
    new_tiling_direction: TilingDirection,
  },
  UserConfigReloaded,
  WindowManaged {
    managed_window: WindowContainer,
  },
  WindowUnmanaged {
    unmanaged_id: Uuid,
    unmanaged_handle: WindowHandle,
  },
  WorkspaceActivated {
    activated_workspace: Workspace,
  },
  WorkspaceDeactivated {
    deactivated_id: Uuid,
    deactivated_name: String,
  },
  WorkingAreaResized {
    affected_monitor: Monitor,
  },
}
