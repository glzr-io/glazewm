use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use tokio::net::TcpStream;
use tokio_tungstenite::{
  connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

use crate::ipc_server::DEFAULT_IPC_PORT;

/// Utility struct for partially deserializing server messages.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialServerMessage {
  pub client_message: Option<String>,
  pub data: Box<RawValue>,
  pub error: Option<String>,
  pub message_type: String,
  pub subscription_id: Option<Uuid>,
  pub success: bool,
}

pub struct IpcClient {
  stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl IpcClient {
  pub async fn connect() -> anyhow::Result<Self> {
    let server_addr = format!("ws://127.0.0.1:{}", DEFAULT_IPC_PORT);

    let (stream, _) = connect_async(server_addr)
      .await
      .context("Failed to connect to IPC server.")?;

    Ok(Self { stream })
  }

  /// Sends a message to the IPC server.
  pub async fn send(&mut self, message: &str) -> anyhow::Result<()> {
    self
      .stream
      .send(Message::Text(message.to_string()))
      .await
      .context("Failed to send command.")?;

    Ok(())
  }

  /// Waits and returns the next reply from the IPC server.
  pub async fn next_response(
    &mut self,
  ) -> anyhow::Result<PartialServerMessage> {
    let response = self
      .stream
      .next()
      .await
      .context("Failed to receive response.")?
      .context("Invalid response message.")?;

    let json_response =
      serde_json::from_str::<PartialServerMessage>(response.to_text()?)?;

    Ok(json_response)
  }

  pub async fn client_response(
    &mut self,
    client_message: &str,
  ) -> Option<PartialServerMessage> {
    while let Ok(response) = self.next_response().await {
      if response.client_message == Some(client_message.to_string()) {
        return Some(response);
      }
    }

    None
  }

  pub async fn event_subscription(
    &mut self,
    subscription_id: &Uuid,
  ) -> Option<PartialServerMessage> {
    while let Ok(response) = self.next_response().await {
      if response.subscription_id == Some(*subscription_id) {
        return Some(response);
      }
    }

    None
  }
}
