use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use ipc_client::DEFAULT_IPC_ADDR;
use tokio::{
  net::TcpListener,
  sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
  },
};
use tokio_tungstenite::accept_async;
use tracing::info;

use crate::{wm_command::WmCommand, wm_event::WmEvent, wm_state::WmState};

#[derive(Debug)]
pub enum IpcMessage {
  Monitors,
  Windows,
  InvokeCommand,
  Subscribe,
}

pub struct IpcServer {
  pub message_rx: UnboundedReceiver<IpcMessage>,
  pub wm_command_rx: UnboundedReceiver<WmCommand>,
}

impl IpcServer {
  pub async fn start() -> Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel::<IpcMessage>();

    let (wm_command_tx, wm_command_rx) =
      mpsc::unbounded_channel::<WmCommand>();

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

    Ok(Self {
      message_rx,
      wm_command_rx,
    })
  }

  pub async fn stop(&self) {
    todo!()
  }

  pub async fn process_message(
    &self,
    _message: IpcMessage,
    wm_state: Arc<Mutex<WmState>>,
  ) {
    todo!()
  }

  pub async fn process_event(&mut self, event: WmEvent) {
    todo!()
  }
}
