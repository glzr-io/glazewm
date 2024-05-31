use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerType {
  Root,
  Monitor,
  Workspace,
  Split,
  Window,
}
