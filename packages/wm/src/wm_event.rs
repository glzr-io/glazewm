use uuid::Uuid;

use crate::{
  common::{platform::WindowHandle, TilingDirection},
  containers::Container,
};

pub struct BindingModeChangedEvent {
  pub new_binding_mode: String,
}

pub struct FocusChangedEvent {
  pub focused_container: Container,
}

pub struct FocusedContainerMovedEvent {
  pub focused_container: Container,
}

pub struct MonitorAddedEvent {
  pub added_monitor: Container,
}

pub struct MonitorRemovedEvent {
  pub removed_id: Uuid,
  pub removed_device_name: String,
}

pub struct NativeFocusSyncedEvent {
  pub focused_container: Container,
}

pub struct TilingDirectionChangedEvent {
  pub new_tiling_direction: TilingDirection,
}

pub struct WindowManagedEvent {
  pub managed_window: Container,
}

pub struct WindowUnmanagedEvent {
  pub unmanaged_id: Uuid,
  pub unmanaged_handle: WindowHandle,
}

pub struct WorkspaceActivatedEvent {
  pub activated_workspace: Container,
}

pub struct WorkspaceDeactivatedEvent {
  pub deactivated_id: Uuid,
  pub deactivated_name: String,
}

pub struct WorkingAreaResizedEvent {
  pub affected_monitor: Container,
}

#[derive(Debug)]
pub enum WmEvent {
  BindingModeChanged(BindingModeChangedEvent),
  FocusChanged(FocusChangedEvent),
  FocusedContainerMoved(FocusedContainerMovedEvent),
  MonitorAdded(MonitorAddedEvent),
  MonitorRemoved(MonitorRemovedEvent),
  NativeFocusSynced(NativeFocusSyncedEvent),
  TilingDirectionChanged(TilingDirectionChangedEvent),
  UserConfigReloaded,
  WindowManaged(WindowManagedEvent),
  WindowUnmanaged(WindowUnmanagedEvent),
  WorkspaceActivated(WorkspaceActivatedEvent),
  WorkspaceDeactivated(WorkspaceDeactivatedEvent),
  WorkingAreaResized(WorkingAreaResizedEvent),
}
