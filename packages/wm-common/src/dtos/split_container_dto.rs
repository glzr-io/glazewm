use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;
use crate::TilingDirection;

/// User-friendly representation of a split container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitContainerDto {
  id: Uuid,
  parent_id: Option<Uuid>,
  children: Vec<ContainerDto>,
  child_focus_order: Vec<Uuid>,
  has_focus: bool,
  tiling_size: f32,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  tiling_direction: TilingDirection,
}
