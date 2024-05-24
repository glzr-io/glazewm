use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{mpsc, Mutex},
  task,
};
use tokio_tungstenite::{
  accept_async, tungstenite::Message, WebSocketStream,
};
use tracing::info;
use uuid::Uuid;

use crate::{
  app_command::{AppCommand, InvokeCommand},
  wm_event::WmEvent,
  wm_state::WmState,
};

pub const DEFAULT_IPC_ADDR: &'static str = "127.0.0.1:6123";

#[derive(Debug, Serialize)]
struct ServerMessage<T> {
  success: bool,
  message_type: ServerMessageType,
  data: Option<T>,
  error: Option<String>,
}

#[derive(Debug, Serialize)]
enum ServerMessageType {
  ClientResponse,
  EventSubscription,
}

#[derive(Debug)]
struct EventSubscription {
  subscription_id: String,
  stream: WebSocketStream<TcpStream>,
}

pub struct IpcServer {
  pub message_rx: mpsc::UnboundedReceiver<AppCommand>,
  pub wm_command_rx:
    mpsc::UnboundedReceiver<(InvokeCommand, Option<Uuid>)>,
  abort_handle: task::AbortHandle,
  // event_subscriptions: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
}

impl IpcServer {
  pub async fn start() -> Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (wm_command_tx, wm_command_rx) = mpsc::unbounded_channel();

    let server = TcpListener::bind(DEFAULT_IPC_ADDR).await?;
    info!("IPC server started on port {}.", DEFAULT_IPC_ADDR);

    // Hashmap of event names and connections subscribed to that event.
    let event_subscriptions = Arc::new(Mutex::new(HashMap::new()));

    let task = task::spawn(async move {
      while let Ok((stream, _)) = server.accept().await {
        task::spawn(Self::handle_connection(
          stream,
          event_subscriptions.clone(),
        ));
      }
    });

    Ok(Self {
      message_rx,
      wm_command_rx,
      abort_handle: task.abort_handle(),
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    event_subscriptions: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
  ) -> anyhow::Result<()> {
    // let client_id = Uuid::new_v4().to_string();
    // info!("Received new IPC connection with client id: {}", client_id);
    info!("Received new IPC connection.");

    let mut subscriptions = event_subscriptions.lock().await;
    let mut ws_stream = accept_async(stream).await?;

    while let Some(Ok(msg)) = ws_stream.next().await {
      if msg.is_text() || msg.is_binary() {
        let app_command =
          AppCommand::try_parse_from(msg.to_text()?.split(" "))?;

        let res = match app_command {
          AppCommand::Start {
            config_path,
            verbosity,
          } => todo!(),
          AppCommand::Query { command } => todo!(),
          AppCommand::Cmd {
            context_container_id,
            command,
          } => todo!(),
        };

        // Respond to the client with the result of the command.
        let response_msg = Message::Text(serde_json::to_string(&res)?);
        ws_stream.send(response_msg).await?;
      }
    }

    // Remove event subscription on websocket disconnect.
    for (_, event_subscriptions) in subscriptions.iter_mut() {
      // event_subscriptions.retain(|subscription| {
      //   // Remove the subscription associated with the disconnected websocket
      //   // You'll need to modify this based on how you track the websocket connection
      //   true
      // });
    }

    Ok(())
  }

  pub async fn process_message(
    &self,
    _message: AppCommand,
    wm_state: Arc<Mutex<WmState>>,
  ) {
    // TODO: Spawn a task so that it doesn't block main thread execution.
  }

  pub async fn process_event(&mut self, event: WmEvent) {
    // // TODO: Spawn a task so that it doesn't block main thread execution.
    // let subscriptions = self.event_subscriptions.lock().await;

    // if let Some(event_subscriptions) = subscriptions.get(event_name) {
    //   for subscription in event_subscriptions {
    //     let socket = subscription.socket.clone();
    //     let event_message = ServerMessage {
    //       success: true,
    //       message_type: "EventSubscription".to_string(),
    //       data: Some(event_data.clone()),
    //       error: None,
    //     };
    //     let message =
    //       Message::Text(serde_json::to_string(&event_message).unwrap());
    //     tokio::spawn(async move {
    //       let mut socket = socket.lock().await;
    //       socket.send(message).await.expect("Failed to send event");
    //     });
    //   }
    // }
  }

  pub fn stop(&self) {
    self.abort_handle.abort();
  }
}

impl Drop for IpcServer {
  fn drop(&mut self) {
    self.stop();
  }
}
