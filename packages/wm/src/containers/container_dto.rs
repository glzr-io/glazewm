use serde::{Deserialize, Serialize};

use crate::{
  monitors::MonitorDto, windows::WindowDto, workspaces::WorkspaceDto,
};

use super::{RootContainerDto, SplitContainerDto};

/// User-friendly representation of a container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContainerDto {
  Root(RootContainerDto),
  Monitor(MonitorDto),
  Workspace(WorkspaceDto),
  Split(SplitContainerDto),
  Window(WindowDto),
}
