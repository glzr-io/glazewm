use anyhow::Context;
use tracing::info;
use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{ShowWindowAsync, SW_MINIMIZE},
};

use wm::cleanup::cleanup_windows;
use wm::containers::ContainerDto;
use wm::ipc_client::IpcClient;
use wm::ipc_server::ClientResponseData;
use wm::wm_event::WmEvent;

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

  let mut managed_handles = Vec::new();

  loop {
    let event_data = client
      .event_subscription(&subscription_id)
      .await
      .and_then(|event| event.data);

    match event_data {
      Some(WmEvent::WindowManaged { managed_window }) => {
        if let ContainerDto::Window(window) = managed_window {
          info!("Watcher added handle: {}.", window.handle);
          managed_handles.push(window.handle);
        }
      }
      Some(WmEvent::WindowUnmanaged {
        unmanaged_handle, ..
      }) => {
        info!("Watcher removed handle: {}.", unmanaged_handle);
        managed_handles.retain(|&handle| handle != unmanaged_handle);
      }
      Some(_) => unreachable!(),
      None => {
        info!("Watcher event subscription ended. Running cleanup.");
        break;
      }
    }
  }

  info!(
    "Cleanup: Remaining managed window handles: {:?}",
    managed_handles
  );

  cleanup_windows(managed_handles);

  Ok(())
}
