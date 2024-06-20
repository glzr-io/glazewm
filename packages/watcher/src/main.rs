use anyhow::Context;

use wm::ipc_client::IpcClient;
use wm::ipc_server::{ClientResponseData, ServerMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut client = IpcClient::connect().await?;

  let subscription_message =
    "subscribe -e window_managed window_unmanaged";

  client
    .send(&subscription_message)
    .await
    .context("Failed to send command to IPC server.")?;

  let client_response = client
    .client_response(&subscription_message)
    .await
    .context("Failed to receive response from IPC server.")?;

  match listen_events(&mut client, &client_response).await {
    Ok(_) => {}
    Err(err) => eprintln!("{}", err),
  }

  Ok(())
}

async fn listen_events(
  ipc_client: &mut IpcClient,
  client_response: &ServerMessage,
) -> anyhow::Result<()> {
  if let ServerMessage::ClientResponse(client_response) = client_response {
    match &client_response.data {
      // For event subscriptions, omit the initial response message and
      // continuously output subsequent event messages.
      Some(ClientResponseData::EventSubscribe(data)) => loop {
        let event_subscription = ipc_client
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
  }
  Ok(())
}
