use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use ipc_client::DEFAULT_IPC_ADDR;
use tokio::{
  net::TcpListener,
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::accept_async;
use tracing::info;

use crate::wm_state::WmState;

pub enum IpcMessage {
  Monitors,
  Windows,
  InvokeCommand,
  Subscribe,
}

pub struct IpcServer {
  message_tx: UnboundedSender<IpcMessage>,
  message_rx: UnboundedReceiver<IpcMessage>,
}

impl IpcServer {
  pub fn new() -> Self {
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<i32>();
    Self {
      message_tx,
      message_rx,
    }
  }

  pub async fn start(&self) -> Result<()> {
    let server = TcpListener::bind(DEFAULT_IPC_ADDR).await?;

    while let Ok((stream, _)) = server.accept().await {
      let mut ws_stream = accept_async(stream).await?;
      info!("Received new IPC connection.");

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

  pub async fn process_message(
    &self,
    _message: IpcMessage,
    wm_state: WmState,
  ) {
    todo!()
  }
}
