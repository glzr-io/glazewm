use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use anyhow::Context;
use clap::Parser;
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryStreamExt};
use serde::Serialize;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{mpsc, Mutex},
  task,
};
use tokio_tungstenite::{
  accept_async, tungstenite::Message, WebSocketStream,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
  app_command::{AppCommand, InvokeCommand},
  wm_event::WmEvent,
  wm_state::WmState,
};

const DEFAULT_IPC_PORT: u32 = 6123;

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
  pub message_rx:
    mpsc::UnboundedReceiver<(AppCommand, mpsc::UnboundedSender<String>)>,
  pub wm_command_rx:
    mpsc::UnboundedReceiver<(InvokeCommand, Option<Uuid>)>,
  abort_handle: task::AbortHandle,
  // /// Hashmap of event names and connections subscribed to that event.
  // event_subs: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
}

impl IpcServer {
  pub async fn start() -> anyhow::Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (wm_command_tx, wm_command_rx) = mpsc::unbounded_channel();

    let server_addr = format!("127.0.0.1:{}", DEFAULT_IPC_PORT);
    let server = TcpListener::bind(server_addr.clone()).await?;
    info!("IPC server started on: '{}'.", server_addr);

    let event_subs = Arc::new(Mutex::new(HashMap::new()));

    let task = task::spawn(async move {
      while let Ok((stream, addr)) = server.accept().await {
        task::spawn(Self::handle_connection(
          stream,
          addr,
          event_subs.clone(),
          message_tx.clone(),
        ));
      }
    });

    Ok(Self {
      message_rx,
      wm_command_rx,
      abort_handle: task.abort_handle(),
      // event_subs: event_subs.clone(),
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    event_subs: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
    message_tx: mpsc::UnboundedSender<(
      AppCommand,
      mpsc::UnboundedSender<String>,
    )>,
  ) -> anyhow::Result<()> {
    info!("Incoming IPC connection from: {}.", addr);

    let ws_stream = accept_async(stream)
      .await
      .context("Error during websocket handshake.")?;

    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    let (mut outgoing, mut incoming) = ws_stream.split();

    loop {
      tokio::select! {
          Some(response) = response_rx.recv() => {
              if let Err(e) = outgoing.send(Message::Text(response)).await {
                  error!("Error sending response: {}", e);
                  break;
              }
          }
          Some(msg) = incoming.next() => {
            let msg = msg.context("Error reading next websocket message.")?;

            if msg.is_text() || msg.is_binary() {
              let app_command =
                AppCommand::try_parse_from(msg.to_text()?.split(" "))?;

              message_tx.send((app_command, response_tx.clone()))?;
            }
          }
      }
    }

    // let receive_from_others = response_rx.map(Ok).forward(outgoing);

    // let get_incoming = incoming
    //   .try_filter(|msg| future::ready(msg.is_text()))
    //   .try_for_each(|msg| {
    //     let tx = response_tx.clone();

    //     // tx.
    //     async move { self.handle_message(tx, msg).await }
    //   });

    // pin_mut!(get_incoming, receive_from_others);

    // while let Some(msg) = ws_stream.next().await {
    //   let msg = msg.context("Error reading next websocket message.")?;

    //   if msg.is_text() || msg.is_binary() {
    //     let app_command =
    //       AppCommand::try_parse_from(msg.to_text()?.split(" "))?;

    //     // message_tx.send((app_command, ws_stream))?;
    //     message_tx.send((app_command))?;
    //   }
    // }

    // TODO: Clean-up event subscriptions on errors.
    let mut subscriptions = event_subs.lock().await;

    // Remove event subscription on websocket disconnect.
    for (_, event_subs) in subscriptions.iter_mut() {
      // event_subs.retain(|subscription| {
      //   // Remove the subscription associated with the disconnected websocket
      //   // You'll need to modify this based on how you track the websocket connection
      //   true
      // });
    }

    info!("IPC disconnection from: {}.", addr);

    Ok(())
  }

  pub async fn process_message(
    &self,
    app_command: AppCommand,
    response_tx: mpsc::UnboundedSender<String>,
    // ws_stream: WebSocketStream<TcpStream>,
    state: Arc<Mutex<WmState>>,
  ) -> anyhow::Result<()> {
    // TODO: Spawn a task so that it doesn't block main thread execution.

    let response = match app_command {
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
    let response_msg = Message::Text(serde_json::to_string(&response)?);
    // response_tx.send(response_msg)?;
    response_tx.send("aaaa".to_string())?;
    // ws_stream.send(response_msg).await?;
  }

  pub async fn process_event(
    &mut self,
    event: WmEvent,
    state: Arc<Mutex<WmState>>,
  ) -> anyhow::Result<()> {
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
    Ok(())
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
