use std::{collections::HashMap, iter, net::SocketAddr, sync::Arc};

use anyhow::Context;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{mpsc, Mutex},
  task,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
  app_command::{AppCommand, QueryCommand},
  containers::{Container, WindowContainer},
  monitors::Monitor,
  user_config::UserConfig,
  wm::WindowManager,
  wm_event::WmEvent,
  workspaces::Workspace,
};

pub const DEFAULT_IPC_PORT: u32 = 6123;

#[derive(Debug, Serialize)]
#[serde(tag = "message_type", rename_all = "snake_case")]
pub enum ServerMessage {
  ClientResponse(ClientResponseMessage),
  EventSubscription(EventSubscriptionMessage),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponseMessage {
  client_message: String,
  data: Option<ClientResponseData>,
  error: Option<String>,
  success: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ClientResponseData {
  Windows(Vec<WindowContainer>),
  Monitors(Vec<Monitor>),
  Workspaces(Vec<Workspace>),
  BindingModes(Vec<String>),
  Focused(Option<Container>),
  Command(CommandResponseData),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponseData {
  subject_container_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubscriptionMessage {
  data: Option<WmEvent>,
  error: Option<String>,
  subscription_id: String,
  success: bool,
}

#[derive(Debug)]
struct EventSubscription {
  subscription_id: String,
  // stream: mpsc::UnboundedSender<EventSubscriptionMessage>,
}

pub struct IpcServer {
  pub message_rx:
    mpsc::UnboundedReceiver<(String, mpsc::UnboundedSender<Message>)>,
  abort_handle: task::AbortHandle,
  // /// Hashmap of event names and connections subscribed to that event.
  // event_subs: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
}

impl IpcServer {
  pub async fn start() -> anyhow::Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();

    let server_addr = format!("127.0.0.1:{}", DEFAULT_IPC_PORT);
    let server = TcpListener::bind(server_addr.clone()).await?;
    info!("IPC server started on: '{}'.", server_addr);

    let event_subs = Arc::new(Mutex::new(HashMap::new()));

    let task = task::spawn(async move {
      while let Ok((stream, addr)) = server.accept().await {
        let event_subs = event_subs.clone();
        let message_tx = message_tx.clone();

        task::spawn(async move {
          if let Err(err) = Self::handle_connection(
            stream,
            addr,
            event_subs.clone(),
            message_tx.clone(),
          )
          .await
          {
            error!("Error handling connection: {}", err);
          }
        });
      }
    });

    Ok(Self {
      message_rx,
      abort_handle: task.abort_handle(),
      // event_subs: event_subs.clone(),
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    event_subs: Arc<Mutex<HashMap<Uuid, EventSubscription>>>,
    message_tx: mpsc::UnboundedSender<(
      String,
      mpsc::UnboundedSender<Message>,
    )>,
  ) -> anyhow::Result<()> {
    info!("Incoming IPC connection from: {}.", addr);

    let ws_stream = accept_async(stream)
      .await
      .context("Error during websocket handshake.")?;

    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    // let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded_channel();
    let (mut outgoing, mut incoming) = ws_stream.split();

    loop {
      tokio::select! {
        Some(response) = response_rx.recv() => {
          if let Err(err) = outgoing.send(response).await {
            error!("Error sending response: {}", err);
            break;
          }
        }
        message = incoming.next() => {
          let message = message.unwrap().context("Could not read next websocket message.")?;

          if message.is_text() || message.is_binary() {
            message_tx.send((message.to_text()?.to_owned(), response_tx.clone()))?;
          }
        }
      }
    }

    // TODO: Clean-up event subscriptions on errors.
    info!("IPC disconnection from: {}.", addr);
    let mut subscriptions = event_subs.lock().await;

    // Remove event subscription on websocket disconnect.
    for (_, event_subs) in subscriptions.iter_mut() {
      // event_subs.retain(|subscription| {
      //   // Remove the subscription associated with the disconnected websocket
      //   // You'll need to modify this based on how you track the websocket connection
      //   true
      // });
    }

    Ok(())
  }

  pub fn process_message(
    &self,
    message: String,
    response_tx: mpsc::UnboundedSender<Message>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let app_command = AppCommand::try_parse_from(
      iter::once("").chain(message.split_whitespace()),
    );

    // TODO: Handle subscribe messages.
    let response = match app_command {
      Ok(AppCommand::Query { command }) => match command {
        QueryCommand::Windows => {
          Ok(ClientResponseData::Windows(wm.state.windows()))
        }
        QueryCommand::Workspaces => {
          Ok(ClientResponseData::Workspaces(wm.state.workspaces()))
        }
        QueryCommand::Monitors => {
          Ok(ClientResponseData::Monitors(wm.state.monitors()))
        }
        QueryCommand::BindingModes => Ok(
          ClientResponseData::BindingModes(wm.state.binding_modes.clone()),
        ),
        QueryCommand::Focused => {
          Ok(ClientResponseData::Focused(wm.state.focused_container()))
        }
      },
      Ok(AppCommand::Cmd {
        subject_container_id,
        command,
      }) => wm
        .process_commands(vec![command], subject_container_id, config)
        .map(|subject_container_id| {
          ClientResponseData::Command(CommandResponseData {
            subject_container_id,
          })
        }),
      Ok(AppCommand::Subscribe { events }) => {
        todo!()
      }
      Err(err) => Err(anyhow::anyhow!(err)),
      _ => Err(anyhow::anyhow!("Unsupported IPC command.")),
    };

    let error = response.as_ref().err().map(|err| err.to_string());
    let success = response.as_ref().is_ok();

    let response = ClientResponseMessage {
      client_message: message,
      data: response.ok(),
      error,
      success,
    };

    // Respond to the client with the result of the command.
    let response_msg = Message::Text(serde_json::to_string(&response)?);
    response_tx.send(response_msg)?;

    Ok(())
  }

  pub fn process_event(
    &mut self,
    event: WmEvent,
    wm: &mut WindowManager,
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
