use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ActiveDrag, DisplayState, Rect, RectDelta, WindowState};

/// User-friendly representation of a tiling or non-tiling window.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowDto {
  pub id: Uuid,
  pub parent_id: Option<Uuid>,
  pub has_focus: bool,
  pub tiling_size: Option<f32>,
  pub width: i32,
  pub height: i32,
  pub x: i32,
  pub y: i32,
  pub state: WindowState,
  pub prev_state: Option<WindowState>,
  pub display_state: DisplayState,
  pub border_delta: RectDelta,
  pub floating_placement: Rect,
  pub handle: crate::WindowHandle,
  pub title: String,
  pub class_name: String,
  pub process_name: String,
  pub active_drag: Option<ActiveDrag>,
}
