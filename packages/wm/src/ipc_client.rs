use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

use crate::ipc_server::DEFAULT_IPC_PORT;

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
  pub async fn send(&mut self, message: String) -> anyhow::Result<()> {
    self
      .stream
      .send(Message::Text(message))
      .await
      .context("Failed to send command.")?;

    Ok(())
  }

  /// Waits and returns the next reply from the IPC server.
  pub async fn next_reply(&mut self) -> anyhow::Result<String> {
    let response = self
      .stream
      .next()
      .await
      .context("Failed to receive response.")?
      .context("Invalid response message.")?;

    let response_str = response.to_text()?;
    Ok(response_str.to_owned())
  }
}
