#![allow(clippy::missing_errors_doc)]

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;
use wm_common::{
  ClientResponseMessage, EventSubscriptionMessage, ServerMessage,
  DEFAULT_IPC_PORT,
};

pub struct IpcClient {
  stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl IpcClient {
  pub async fn connect() -> anyhow::Result<Self> {
    let server_addr = format!("ws://127.0.0.1:{DEFAULT_IPC_PORT}");

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
  pub async fn next_message(&mut self) -> anyhow::Result<ServerMessage> {
    let response = self
      .stream
      .next()
      .await
      .context("Failed to receive response.")?
      .context("Invalid response message.")?;

    let json_response =
      serde_json::from_str::<ServerMessage>(response.to_text()?)?;

    Ok(json_response)
  }

  pub async fn client_response(
    &mut self,
    client_message: &str,
  ) -> Option<ClientResponseMessage> {
    while let Ok(response) = self.next_message().await {
      if let ServerMessage::ClientResponse(client_response) = response {
        if client_response.client_message == client_message {
          return Some(client_response);
        }
      }
    }

    None
  }

  pub async fn event_subscription(
    &mut self,
    subscription_id: &Uuid,
  ) -> Option<EventSubscriptionMessage> {
    while let Ok(response) = self.next_message().await {
      if let ServerMessage::EventSubscription(event_sub) = response {
        if &event_sub.subscription_id == subscription_id {
          return Some(event_sub);
        }
      }
    }

    None
  }
}
