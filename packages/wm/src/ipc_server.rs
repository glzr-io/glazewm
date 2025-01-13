use std::{iter, net::SocketAddr};

use anyhow::{bail, Context};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{broadcast, mpsc},
  task,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{info, warn};
use uuid::Uuid;
use wm_common::{
  AppCommand, AppMetadataData, BindingModesData, ClientResponseData,
  ClientResponseMessage, CommandData, EventSubscribeData,
  EventSubscriptionMessage, FocusedData, MonitorsData, QueryCommand,
  ServerMessage, SubscribableEvent, TilingDirectionData, WindowsData,
  WmEvent, WorkspacesData, DEFAULT_IPC_PORT,
};

use crate::{
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm::WindowManager,
};

pub struct IpcServer {
  abort_handle: task::AbortHandle,
  pub message_rx: mpsc::UnboundedReceiver<(
    String,
    mpsc::UnboundedSender<Message>,
    broadcast::Sender<()>,
  )>,
  _event_rx: broadcast::Receiver<(SubscribableEvent, WmEvent)>,
  event_tx: broadcast::Sender<(SubscribableEvent, WmEvent)>,
  _unsubscribe_rx: broadcast::Receiver<Uuid>,
  unsubscribe_tx: broadcast::Sender<Uuid>,
}

impl IpcServer {
  pub async fn start() -> anyhow::Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (event_tx, _event_rx) = broadcast::channel(16);
    let (unsubscribe_tx, _unsubscribe_rx) = broadcast::channel(16);

    let server_addr = format!("127.0.0.1:{DEFAULT_IPC_PORT}");
    let server = TcpListener::bind(server_addr.clone()).await?;
    info!("IPC server started on: '{}'.", server_addr);

    let task = task::spawn(async move {
      while let Ok((stream, addr)) = server.accept().await {
        let message_tx = message_tx.clone();

        task::spawn(async move {
          if let Err(err) =
            Self::handle_connection(stream, addr, message_tx).await
          {
            warn!("Error handling connection: {}", err);
          }
        });
      }
    });

    Ok(Self {
      abort_handle: task.abort_handle(),
      #[allow(clippy::used_underscore_binding)]
      _event_rx,
      event_tx,
      message_rx,
      unsubscribe_tx,
      #[allow(clippy::used_underscore_binding)]
      _unsubscribe_rx,
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    message_tx: mpsc::UnboundedSender<(
      String,
      mpsc::UnboundedSender<Message>,
      broadcast::Sender<()>,
    )>,
  ) -> anyhow::Result<()> {
    info!("Incoming IPC connection from: {}.", addr);

    let ws_stream = accept_async(stream)
      .await
      .context("Error during websocket handshake.")?;

    let (mut outgoing, mut incoming) = ws_stream.split();
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    let (disconnection_tx, _) = broadcast::channel(16);

    loop {
      tokio::select! {
        Some(response) = response_rx.recv() => {
          if let Err(err) = outgoing.send(response).await {
            warn!("Error sending response: {}", err);
            break;
          }
        }
        message = incoming.next() => {
          if let Some(Ok(message)) = message {
            if message.is_text() || message.is_binary() {
              message_tx.send((
                message.to_text()?.to_owned(),
                response_tx.clone(),
                disconnection_tx.clone(),
              ))?;
            }
          } else {
            warn!("Could not read next websocket message.");
            break;
          }
        }
      }
    }

    info!("IPC disconnection from: {}.", addr);
    disconnection_tx.send(())?;

    Ok(())
  }

  pub fn process_message(
    &self,
    message: String,
    response_tx: &mpsc::UnboundedSender<Message>,
    disconnection_tx: &broadcast::Sender<()>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let app_command = AppCommand::try_parse_from(
      iter::once("").chain(message.split_whitespace()),
    );

    let response_data =
      app_command
        .map_err(anyhow::Error::msg)
        .and_then(|app_command| {
          self.handle_app_command(
            app_command,
            response_tx,
            disconnection_tx,
            wm,
            config,
          )
        });

    // Respond to the client with the result of the command.
    response_tx
      .send(Self::to_client_response_msg(message, response_data)?)?;

    Ok(())
  }

  #[allow(clippy::too_many_lines)]
  fn handle_app_command(
    &self,
    app_command: AppCommand,
    response_tx: &mpsc::UnboundedSender<Message>,
    disconnection_tx: &broadcast::Sender<()>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<ClientResponseData> {
    let response_data = match app_command {
      AppCommand::Query { command } => match command {
        QueryCommand::Windows => {
          ClientResponseData::Windows(WindowsData {
            windows: wm
              .state
              .windows()
              .into_iter()
              .map(|window| window.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::Workspaces => {
          ClientResponseData::Workspaces(WorkspacesData {
            workspaces: wm
              .state
              .workspaces()
              .into_iter()
              .map(|workspace| workspace.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::Monitors => {
          ClientResponseData::Monitors(MonitorsData {
            monitors: wm
              .state
              .monitors()
              .into_iter()
              .map(|monitor| monitor.to_dto())
              .try_collect()?,
          })
        }
        QueryCommand::BindingModes => {
          ClientResponseData::BindingModes(BindingModesData {
            binding_modes: wm.state.binding_modes.clone(),
          })
        }
        QueryCommand::Focused => {
          let focused_container = wm
            .state
            .focused_container()
            .context("No focused container.")?;

          ClientResponseData::Focused(FocusedData {
            focused: focused_container.to_dto()?,
          })
        }
        QueryCommand::AppMetadata => {
          ClientResponseData::AppMetadata(AppMetadataData {
            version: env!("VERSION_NUMBER").to_string(),
          })
        }
        QueryCommand::TilingDirection => {
          let direction_container = wm
            .state
            .focused_container()
            .and_then(|focused| focused.direction_container())
            .context("No direction container.")?;

          ClientResponseData::TilingDirection(TilingDirectionData {
            direction_container: direction_container.to_dto()?,
            tiling_direction: direction_container.tiling_direction(),
          })
        }
        QueryCommand::Paused => {
          ClientResponseData::Paused(wm.state.is_paused)
        }
      },
      AppCommand::Command {
        subject_container_id,
        command,
      } => {
        let subject_container_id = wm.process_commands(
          &vec![command],
          subject_container_id,
          config,
        )?;

        ClientResponseData::Command(CommandData {
          subject_container_id,
        })
      }
      AppCommand::Sub { events } => {
        let subscription_id = Uuid::new_v4();
        info!("New event subscription {}: {:?}", subscription_id, events);

        let response_tx = response_tx.clone();
        let mut event_rx = self.event_tx.subscribe();
        let mut unsubscribe_rx = self.unsubscribe_tx.subscribe();
        let mut disconnection_rx = disconnection_tx.subscribe();

        task::spawn(async move {
          loop {
            tokio::select! {
              Ok(()) = disconnection_rx.recv() => {
                break;
              }
              Ok(id) = unsubscribe_rx.recv() => {
                if id == subscription_id {
                  break;
                }
              }
              Ok((event_type, event)) = event_rx.recv() => {
                // Check whether the event is one of the subscribed events.
                if events.contains(&event_type)
                  || events.contains(&SubscribableEvent::All)
                {
                  let res = Self::to_event_subscription_msg(
                    subscription_id,
                    event,
                  )
                  .map(|event_msg| response_tx.send(event_msg));

                  if let Err(err) = res {
                    warn!("Error emitting WM event: {}", err);
                    break;
                  }
                }
              }
            }
          }
        });

        ClientResponseData::EventSubscribe(EventSubscribeData {
          subscription_id,
        })
      }
      AppCommand::Unsub { subscription_id } => {
        self
          .unsubscribe_tx
          .send(subscription_id)
          .context("Failed to unsubscribe from event.")?;

        ClientResponseData::EventUnsubscribe
      }
      AppCommand::Start { .. } => bail!("Unsupported IPC command."),
    };

    Ok(response_data)
  }

  fn to_client_response_msg(
    client_message: String,
    response_data: anyhow::Result<ClientResponseData>,
  ) -> anyhow::Result<Message> {
    let error = response_data.as_ref().err().map(ToString::to_string);
    let success = response_data.as_ref().is_ok();

    let message = ServerMessage::ClientResponse(ClientResponseMessage {
      client_message,
      data: response_data.ok(),
      error,
      success,
    });

    let message_json = serde_json::to_string(&message)?;
    Ok(Message::Text(message_json))
  }

  fn to_event_subscription_msg(
    subscription_id: Uuid,
    event: WmEvent,
  ) -> anyhow::Result<Message> {
    let message =
      ServerMessage::EventSubscription(EventSubscriptionMessage {
        data: Some(event),
        error: None,
        subscription_id,
        success: true,
      });

    let message_json = serde_json::to_string(&message)?;
    Ok(Message::Text(message_json))
  }

  pub fn process_event(&mut self, event: WmEvent) -> anyhow::Result<()> {
    let event_type = match event {
      WmEvent::ApplicationExiting => SubscribableEvent::ApplicationExiting,
      WmEvent::BindingModesChanged { .. } => {
        SubscribableEvent::BindingModesChanged
      }
      WmEvent::FocusChanged { .. } => SubscribableEvent::FocusChanged,
      WmEvent::FocusedContainerMoved { .. } => {
        SubscribableEvent::FocusedContainerMoved
      }
      WmEvent::MonitorAdded { .. } => SubscribableEvent::MonitorAdded,
      WmEvent::MonitorUpdated { .. } => SubscribableEvent::MonitorUpdated,
      WmEvent::MonitorRemoved { .. } => SubscribableEvent::MonitorRemoved,
      WmEvent::TilingDirectionChanged { .. } => {
        SubscribableEvent::TilingDirectionChanged
      }
      WmEvent::UserConfigChanged { .. } => {
        SubscribableEvent::UserConfigChanged
      }
      WmEvent::WindowManaged { .. } => SubscribableEvent::WindowManaged,
      WmEvent::WindowUnmanaged { .. } => {
        SubscribableEvent::WindowUnmanaged
      }
      WmEvent::WorkspaceActivated { .. } => {
        SubscribableEvent::WorkspaceActivated
      }
      WmEvent::WorkspaceDeactivated { .. } => {
        SubscribableEvent::WorkspaceDeactivated
      }
      WmEvent::WorkspaceUpdated { .. } => {
        SubscribableEvent::WorkspaceUpdated
      }
      WmEvent::PauseChanged { .. } => SubscribableEvent::PauseChanged,
    };

    self.event_tx.send((event_type, event))?;

    Ok(())
  }

  pub fn stop(&self) {
    info!("Shutting down IPC server.");
    self.abort_handle.abort();
  }
}

impl Drop for IpcServer {
  fn drop(&mut self) {
    self.stop();
  }
}
