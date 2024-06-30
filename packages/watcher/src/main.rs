use anyhow::Context;
use tracing::info;

use wm::{
  cleanup::run_cleanup, common::platform::NativeWindow,
  containers::ContainerDto, ipc_client::IpcClient,
  ipc_server::ClientResponseData, wm_event::WmEvent,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt().init();

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

  let mut managed_windows = Vec::new();

  loop {
    let event_data = client
      .event_subscription(&subscription_id)
      .await
      .and_then(|event| event.data);

    match event_data {
      Some(WmEvent::WindowManaged { managed_window }) => {
        if let ContainerDto::Window(window) = managed_window {
          info!("Watcher added handle: {}.", window.handle);
          managed_windows.push(NativeWindow::new(window.handle));
        }
      }
      Some(WmEvent::WindowUnmanaged {
        unmanaged_handle, ..
      }) => {
        info!("Watcher removed handle: {}.", unmanaged_handle);
        managed_windows.retain(|window| window.handle != unmanaged_handle);
      }
      Some(_) => unreachable!(),
      None => {
        info!("Watcher event subscription ended. Running cleanup.");
        break;
      }
    }
  }

  run_cleanup(managed_windows);

  Ok(())
}
