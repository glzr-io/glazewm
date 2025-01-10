use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;
use crate::Rect;

/// User-friendly representation of a monitor.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
  id: Uuid,
  parent_id: Option<Uuid>,
  children: Vec<ContainerDto>,
  child_focus_order: Vec<Uuid>,
  has_focus: bool,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  dpi: u32,
  scale_factor: f32,
  handle: isize,
  device_name: String,
  device_path: Option<String>,
  hardware_id: Option<String>,
  working_rect: Rect,
}
