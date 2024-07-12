use anyhow::{bail, Context};
use tracing::info;
use wm::{
  cleanup::run_cleanup, common::platform::NativeWindow,
  containers::ContainerDto, ipc_client::IpcClient,
  ipc_server::ClientResponseData, windows::WindowDto, wm_event::WmEvent,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt().init();

  let mut client = IpcClient::connect().await?;

  // Get handles to windows that are already open on watcher launch.
  let mut managed_handles = query_initial_windows(&mut client)
    .await?
    .into_iter()
    .map(|window| window.handle)
    .collect::<Vec<_>>();

  // Update window handles on window manage/unmanage events.
  let subscribe_res =
    watch_managed_handles(&mut client, &mut managed_handles).await;

  match subscribe_res {
    Ok(_) => info!("WM exited successfully. Skipping watcher cleanup."),
    Err(_) => {
      info!("WM exitted unexpectedly. Running watcher cleanup.");

      let managed_windows = managed_handles
        .into_iter()
        .map(|handle| NativeWindow::new(handle))
        .collect::<Vec<_>>();

      run_cleanup(managed_windows);
    }
  }

  Ok(())
}

async fn query_initial_windows(
  client: &mut IpcClient,
) -> anyhow::Result<Vec<WindowDto>> {
  let query_message = "query windows";

  client
    .send(query_message)
    .await
    .context("Failed to send window query command.")?;

  client
    .client_response(&query_message)
    .await
    .and_then(|response| match response.data {
      Some(ClientResponseData::Windows(data)) => Some(data),
      _ => None,
    })
    .map(|data| {
      data
        .windows
        .into_iter()
        .filter_map(|container| match container {
          ContainerDto::Window(window) => Some(window),
          _ => None,
        })
        .collect::<Vec<_>>()
    })
    .context("Invalid data in windows query response.")
}

async fn watch_managed_handles(
  client: &mut IpcClient,
  handles: &mut Vec<isize>,
) -> anyhow::Result<()> {
  let subscription_message =
    "subscribe -e window_managed window_unmanaged application_exiting";

  client
    .send(&subscription_message)
    .await
    .context("Failed to send subscribe command to IPC server.")?;

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
          info!("Watcher added handle: {}.", window.handle);
          handles.push(window.handle);
        }
      }
      Some(WmEvent::WindowUnmanaged {
        unmanaged_handle, ..
      }) => {
        info!("Watcher removed handle: {}.", unmanaged_handle);
        handles.retain(|&handle| handle != unmanaged_handle);
      }
      Some(WmEvent::ApplicationExiting) => {
        return Ok(());
      }
      Some(_) => unreachable!(),
      None => {
        bail!("IPC connection closed unexpectedly.")
      }
    }
  }
}
