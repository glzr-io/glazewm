use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::mpsc::UnboundedSender};
use tokio_tungstenite::accept_async;

pub enum IpcMessage {
  Monitors,
  Windows,
  InvokeCommand,
  Subscribe,
}

pub struct IpcServer {
  message_tx: UnboundedSender<IpcMessage>,
}

const DEFAULT_PORT: u32 = 6123;

impl IpcServer {
  pub fn new(message_tx: UnboundedSender<IpcMessage>) -> Self {
    Self { message_tx }
  }

  pub async fn start(&self) -> Result<()> {
    let server = TcpListener::bind("127.0.0.1:6123").await?;

    while let Ok((stream, _)) = server.accept().await {
      let mut ws_stream = accept_async(stream).await?;

      while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
          ws_stream.send(msg).await?;
        }
      }
    }

    Ok(())
  }

  pub async fn stop(&self) {
    todo!()
  }

  pub async fn process_message(&self, _message: IpcMessage, wm_state: WmState) {
    todo!()
  }
}
