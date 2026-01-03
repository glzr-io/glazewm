use serde::{Deserialize, Serialize};

/// Minimal mapping of a native application to a workspace name.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppWorkspaceEntry {
  pub handle: isize,
  pub process_name: Option<String>,
  pub class_name: Option<String>,
  pub title: Option<String>,
  pub workspace: String,
}
