use std::{fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BindingModeConfig, ContainerDto, TilingDirection, WmEvent};

/// Default port for the IPC server.
///
/// The server listens here whenever the port is free, and clients fall
/// back to it when they find no port file.
pub const DEFAULT_IPC_PORT: u16 = 6123;

/// Path of the file that holds the port of this session's IPC server.
///
/// A port is machine-wide, so only the first logged-in user gets
/// [`DEFAULT_IPC_PORT`] and the rest get one picked at runtime. Clients
/// read this file rather than assuming either.
///
/// The session ID is part of the file name because a single user can be
/// logged into more than one session at once.
pub fn ipc_port_file() -> anyhow::Result<PathBuf> {
  let home_dir =
    home::home_dir().context("Unable to get home directory.")?;

  Ok(
    home_dir
      .join(".glzr/glazewm")
      .join(format!("ipc-port-{}", wm_platform::session_id())),
  )
}

/// Publishes the port that the IPC server is listening on.
pub fn write_ipc_port(port: u16) -> anyhow::Result<()> {
  let path = ipc_port_file()?;

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  fs::write(&path, port.to_string())
    .with_context(|| format!("Failed to write '{}'.", path.display()))
}

/// Port that this session's IPC server is listening on.
///
/// Falls back to [`DEFAULT_IPC_PORT`] when the file is missing or
/// unreadable.
#[must_use]
pub fn read_ipc_port() -> u16 {
  ipc_port_file()
    .ok()
    .and_then(|path| fs::read_to_string(path).ok())
    .and_then(|port| port.trim().parse().ok())
    .unwrap_or(DEFAULT_IPC_PORT)
}

/// Removes the port file, so that clients don't try to reach a server that
/// is gone.
pub fn remove_ipc_port_file() {
  if let Ok(path) = ipc_port_file() {
    let _ = fs::remove_file(path);
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "messageType", rename_all = "snake_case")]
pub enum ServerMessage {
  ClientResponse(ClientResponseMessage),
  EventSubscription(EventSubscriptionMessage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponseMessage {
  pub client_message: String,
  pub data: Option<ClientResponseData>,
  pub error: Option<String>,
  pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ClientResponseData {
  AppMetadata(AppMetadataData),
  BindingModes(BindingModesData),
  Command(CommandData),
  EventSubscribe(EventSubscribeData),
  EventUnsubscribe,
  Focused(FocusedData),
  Monitors(MonitorsData),
  TilingDirection(TilingDirectionData),
  Windows(WindowsData),
  Workspaces(WorkspacesData),
  Paused(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppMetadataData {
  pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingModesData {
  pub binding_modes: Vec<BindingModeConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandData {
  pub subject_container_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubscribeData {
  pub subscription_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusedData {
  pub focused: ContainerDto,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorsData {
  pub monitors: Vec<ContainerDto>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TilingDirectionData {
  pub tiling_direction: TilingDirection,
  pub direction_container: ContainerDto,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsData {
  pub windows: Vec<ContainerDto>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesData {
  pub workspaces: Vec<ContainerDto>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubscriptionMessage {
  pub data: Option<WmEvent>,
  pub error: Option<String>,
  pub subscription_id: Uuid,
  pub success: bool,
}
