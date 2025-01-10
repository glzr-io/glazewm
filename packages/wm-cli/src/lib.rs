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

pub async fn start_cli(args: Vec<String>) -> Result<()> {
  let mut client = IpcClient::connect().await?;

  let message = args[1..].join(" ");
  client
    .send(&message)
    .await
    .context("Failed to send command to IPC server.")?;

  let client_response = client
    .client_response(&message)
    .await
    .context("Failed to receive response from IPC server.")?;

  match client_response.data {
    // For event subscriptions, omit the initial response message and
    // continuously output subsequent event messages.
    Some(ClientResponseData::EventSubscribe(data)) => loop {
      let event_subscription = client
        .event_subscription(&data.subscription_id)
        .await
        .context("Failed to receive response from IPC server.")?;

      println!("{}", serde_json::to_string(&event_subscription)?);
    },
    // For all other messages, output and exit when the first response
    // message is received.
    _ => {
      println!("{}", serde_json::to_string(&client_response)?);
    }
  }

  Ok(())
}
