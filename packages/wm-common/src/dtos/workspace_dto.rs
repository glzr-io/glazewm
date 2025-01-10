use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;
use crate::TilingDirection;

/// User-friendly representation of a workspace.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDto {
  id: Uuid,
  name: String,
  display_name: Option<String>,
  parent_id: Option<Uuid>,
  children: Vec<ContainerDto>,
  child_focus_order: Vec<Uuid>,
  has_focus: bool,
  is_displayed: bool,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  tiling_direction: TilingDirection,
}
