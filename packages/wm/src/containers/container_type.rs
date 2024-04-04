use serde::{Deserialize, Serialize};

/// TODO: Not sure whether this type is needed at all. Instead just use
/// Serde enum tag.
#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContainerType {
  Root,
  Monitor,
  Workspace,
  Split,
  Window,
}
