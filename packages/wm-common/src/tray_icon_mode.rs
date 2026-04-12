use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Runtime display mode for the WM tray icon.
#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  Deserialize,
  Eq,
  PartialEq,
  Serialize,
  ValueEnum,
)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TrayIconMode {
  /// Shows the WM status icon variants.
  #[default]
  Status,
  /// Shows the currently focused workspace number.
  Workspace,
}
