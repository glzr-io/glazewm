use std::string;

use anyhow::{bail, Result};
use serde::{Deserialize, Deserializer};

use crate::common::{Direction, TilingDirection};

#[derive(Debug)]
pub enum WmCommand {
  CloseWindow,
  ExitWm,
  FocusInDirection(Direction),
  FocusRecentWorkspace,
  FocusWorkspaceInSequence,
  FocusWorkspace(String),
  IgnoreWindow,
  MoveWindow(Direction),
  MoveWindowToWorkspace(String),
  MoveWorkspace(Direction),
  Noop,
  Redraw,
  ReloadConfig,
  SetTilingDirection(TilingDirection),
  ToggleTilingDirection,
  ToggleFocusMode,
}

impl WmCommand {
  pub fn from_str(unparsed: &str) -> Result<Self> {
    let parts: Vec<&str> = unparsed.split_whitespace().collect();

    let command = match parts.as_slice() {
      ["close_window"] => WmCommand::CloseWindow,
      ["exit_wm"] => WmCommand::ExitWm,
      ["focus", direction] => {
        WmCommand::FocusInDirection(Direction::from_str(direction)?)
      }
      ["focus_workspace", name] => {
        WmCommand::FocusWorkspace(name.to_string())
      }
      ["ignore_window"] => WmCommand::IgnoreWindow,
      ["move_window", direction] => {
        WmCommand::MoveWindow(Direction::from_str(direction)?)
      }
      ["move_window_to_workspace", name] => {
        WmCommand::MoveWindowToWorkspace(name.to_string())
      }
      ["move_workspace", direction] => {
        WmCommand::MoveWorkspace(Direction::from_str(direction)?)
      }
      ["noop"] => WmCommand::Noop,
      ["redraw"] => WmCommand::Redraw,
      ["reload_config"] => WmCommand::ReloadConfig,
      ["set_tiling_direction", tiling_direction] => {
        WmCommand::SetTilingDirection(TilingDirection::from_str(
          tiling_direction,
        )?)
      }
      ["toggle_tiling_direction", "vertical"] => {
        WmCommand::ToggleTilingDirection
      }
      ["toggle_focus_mode"] => WmCommand::ToggleFocusMode,
      _ => bail!("Not a known command."),
    };

    Ok(command)
  }
}

impl<'de> Deserialize<'de> for WmCommand {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let str = String::deserialize(deserializer)?;
    Self::from_str(&str).map_err(serde::de::Error::custom)
  }
}
