use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;
use wm_platform::Rect;

/// User-friendly representation of a monitor.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
  pub id: Uuid,
  pub parent_id: Option<Uuid>,
  pub children: Vec<ContainerDto>,
  pub child_focus_order: Vec<Uuid>,
  pub has_focus: bool,
  pub width: i32,
  pub height: i32,
  pub x: i32,
  pub y: i32,
  pub dpi: u32,
  pub scale_factor: f32,
  // pub handle: isize,
  // pub device_name: String,
  // pub device_path: Option<String>,
  // pub hardware_id: Option<String>,
  pub working_rect: Rect,
}
