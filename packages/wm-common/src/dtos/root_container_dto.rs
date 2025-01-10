use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;

/// User-friendly representation of a root container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootContainerDto {
  id: Uuid,
  parent_id: Option<Uuid>,
  children: Vec<ContainerDto>,
  child_focus_order: Vec<Uuid>,
}
