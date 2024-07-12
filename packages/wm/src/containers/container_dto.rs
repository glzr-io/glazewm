use serde::{Deserialize, Serialize};

use super::{RootContainerDto, SplitContainerDto};
use crate::{
  monitors::MonitorDto, windows::WindowDto, workspaces::WorkspaceDto,
};

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
