use anyhow::Context;

use wm::containers::ContainerDto;
use wm::ipc_client::IpcClient;
use wm::ipc_server::ClientResponseData;
use wm::wm_event::WmEvent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut client = IpcClient::connect().await?;

  let subscription_message =
    "subscribe -e window_managed window_unmanaged";

  client
    .send(&subscription_message)
    .await
    .context("Failed to send command to IPC server.")?;

  let subscription_id = client
    .client_response(&subscription_message)
    .await
    .and_then(|response| match response.data {
      Some(ClientResponseData::EventSubscribe(data)) => {
        Some(data.subscription_id)
      }
      _ => None,
    })
    .context("No subscription ID in watcher event subscription.")?;

  loop {
    let event_data = client
      .event_subscription(&subscription_id)
      .await
      .and_then(|event| event.data);

    match event_data {
      Some(WmEvent::WindowManaged { managed_window }) => {
        if let ContainerDto::Window(window) = managed_window {
          println!("managed handle: {}", window.handle);
          // TODO: Add handle to list of managed handles.
        }
      }
      Some(WmEvent::WindowUnmanaged {
        unmanaged_handle, ..
      }) => {
        println!("unmanaged handle: {}", unmanaged_handle);
        // TODO: Pop handle from list of managed handles.
      }
      Some(_) => unreachable!(),
      None => {
        println!("No event subscription data.");
        break;
      }
    }
  }

  // TODO: Run shared cleanup fn here.

  Ok(())
}
