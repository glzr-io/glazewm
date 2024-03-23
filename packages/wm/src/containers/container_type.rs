use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContainerType {
  Root,
  Monitor,
  Workspace,
  Split,
  Window,
}