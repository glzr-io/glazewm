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
  pub id: Uuid,
  pub parent_id: Option<Uuid>,
  pub children: Vec<ContainerDto>,
  pub child_focus_order: Vec<Uuid>,
  pub has_focus: bool,
  pub tiling_size: f32,
  pub width: i32,
  pub height: i32,
  pub x: i32,
  pub y: i32,
  pub tiling_direction: TilingDirection,
}
