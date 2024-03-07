use uuid::Uuid;

use crate::{
  common::{platform::WindowHandle, TilingDirection},
  containers::ContainerRef,
};

#[derive(Debug)]
pub struct BindingModeChangedEvent {
  pub new_binding_mode: String,
}

#[derive(Debug)]
pub struct FocusChangedEvent {
  pub focused_container: ContainerRef,
}

#[derive(Debug)]
pub struct FocusedContainerMovedEvent {
  pub focused_container: ContainerRef,
}

#[derive(Debug)]
pub struct MonitorAddedEvent {
  pub added_monitor: ContainerRef,
}

#[derive(Debug)]
pub struct MonitorRemovedEvent {
  pub removed_id: Uuid,
  pub removed_device_name: String,
}

#[derive(Debug)]
pub struct NativeFocusSyncedEvent {
  pub focused_container: ContainerRef,
}

#[derive(Debug)]
pub struct TilingDirectionChangedEvent {
  pub new_tiling_direction: TilingDirection,
}

#[derive(Debug)]
pub struct WindowManagedEvent {
  pub managed_window: ContainerRef,
}

#[derive(Debug)]
pub struct WindowUnmanagedEvent {
  pub unmanaged_id: Uuid,
  pub unmanaged_handle: WindowHandle,
}

#[derive(Debug)]
pub struct WorkspaceActivatedEvent {
  pub activated_workspace: ContainerRef,
}

#[derive(Debug)]
pub struct WorkspaceDeactivatedEvent {
  pub deactivated_id: Uuid,
  pub deactivated_name: String,
}

#[derive(Debug)]
pub struct WorkingAreaResizedEvent {
  pub affected_monitor: ContainerRef,
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
