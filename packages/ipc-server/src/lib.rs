use std::net::TcpListener;

use tokio::sync::mpsc::UnboundedSender;
use tungstenite::accept;

pub enum IpcMessage {
  Monitors,
  Windows,
  InvokeCommand,
  Subscribe,
}

pub struct IpcServer {
  message_tx: UnboundedSender<IpcMessage>,
}

impl IpcServer {
  pub fn new(message_tx: UnboundedSender<IpcMessage>) -> Self {
    Self { message_tx }
  }

  pub async fn start(&self) {
    let server = TcpListener::bind("127.0.0.1:6123").unwrap();

    for stream in server.incoming() {
      tokio::spawn(async move {
        let mut websocket = accept(stream.unwrap()).unwrap();
        loop {
          let msg = websocket.read().unwrap();

          // We do not want to send back ping/pong messages.
          if msg.is_binary() || msg.is_text() {
            websocket.send(msg).unwrap();
          }
        }
      });
    }
  }

  pub async fn stop(&self) {
    // Stop the server
  }
}

pub fn start_server() {}
