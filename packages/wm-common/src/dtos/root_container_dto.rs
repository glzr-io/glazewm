use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContainerDto;

/// User-friendly representation of a root container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootContainerDto {
  pub id: Uuid,
  pub parent_id: Option<Uuid>,
  pub children: Vec<ContainerDto>,
  pub child_focus_order: Vec<Uuid>,
}
