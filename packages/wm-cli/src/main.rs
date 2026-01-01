use std::{env, process::Command, fs::File, path::PathBuf};

use anyhow::{Context, Error};
use wm_cli::start;
use wm_common::{AppCommand, ClientResponseData, ContainerDto, WindowDto};
use serde::Deserialize;
use wm_ipc_client::IpcClient;
use wm_platform::Platform;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize)]
struct AppWorkspaceEntry {
  handle: isize,
  process_name: Option<String>,
  class_name: Option<String>,
  title: Option<String>,
  workspace: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();

  // Special helper: restore-mapping reads the mapping JSON from the
  // user's config directory and instructs the WM to move matching
  // windows to their configured workspaces.
  if args.len() > 1 && args[1] == "restore-mapping" {
    return restore_mapping().await;
  }

  let app_command = AppCommand::parse_with_default(&args);

  match app_command {
    AppCommand::Start { .. } => {
      let exe_path = env::current_exe()?;
      let exe_dir = exe_path
        .parent()
        .context("Failed to resolve path to the current executable.")?
        .to_owned();

      // Main executable is either in the current directory (when running
      // debug/release builds) or in the parent directory when packaged.
      let main_path =
        [exe_dir.join("glazewm.exe"), exe_dir.join("../glazewm.exe")]
          .into_iter()
          .find(|path| path.exists() && *path != exe_path)
          .and_then(|path| path.to_str().map(ToString::to_string))
          .context("Failed to resolve path to the main executable.")?;

      // UIAccess applications can't be started directly, so we need to use
      // CMD to start it. The start command is used to avoid a long-running
      // CMD process in the background.
      Command::new("cmd")
        .args(
          ["/C", "start", "", &main_path]
            .into_iter()
            .chain(args.iter().skip(1).map(String::as_str)),
        )
        .spawn()
        .context("Failed to start main executable.")?;

      Ok(())
    }
    _ => start(args).await,
  }
}

async fn restore_mapping() -> Result<(), Error> {
  // Resolve config path (same logic as UserConfig::new).
  let default_config_path = home::home_dir()
    .context("Unable to get home directory.")?
    .join(".glzr/glazewm/config.yaml");

  let config_path = env::var("GLAZEWM_CONFIG_PATH")
    .ok()
    .map(PathBuf::from)
    .unwrap_or(default_config_path);

  let mapping_path = config_path
    .parent()
    .map(|p| p.join("glazewm_apps_workspaces.json"))
    .unwrap_or_else(|| PathBuf::from("glazewm_apps_workspaces.json"));

  if !mapping_path.exists() {
    return Err(anyhow::anyhow!(
      "Mapping file not found: {}",
      mapping_path.display()
    ));
  }

  let file = File::open(&mapping_path)
    .context("Failed to open mapping file.")?;

  let entries: Vec<AppWorkspaceEntry> = serde_json::from_reader(file)
    .context("Failed to parse mapping JSON.")?;

  let mut client = IpcClient::connect().await?;

  // Query current windows.
  let query_cmd = "q windows";
  client.send(query_cmd).await?;

  let resp = client
    .client_response(query_cmd)
    .await
    .context("No response for windows query.")?;

  let windows = match resp.data {
    Some(ClientResponseData::Windows(wd)) => wd.windows,
    _ => vec![],
  };

  // Build a set of handles already managed by WM.
  let mut managed_handles: std::collections::HashSet<isize> = std::collections::HashSet::new();
  for c in &windows {
    if let ContainerDto::Window(wdto) = c {
      managed_handles.insert(wdto.handle);
    }
  }

  // For each entry, try to find a match in WM; if not found, try to
  // find the native window and focus it so WM will manage it, then
  // re-query WM and move it.
  for entry in entries {
    // First try to find in current WM windows by handle/process/title.
    let mut found_id: Option<uuid::Uuid> = None;

    for c in &windows {
      if let ContainerDto::Window(wdto) = c {
        if wdto.handle == entry.handle
          || entry.process_name.as_ref().map(|p| p == &wdto.process_name).unwrap_or(false)
          || entry.title.as_ref().map(|t| wdto.title.contains(t)).unwrap_or(false)
        {
          found_id = Some(wdto.id);
          break;
        }
      }
    }

    if let Some(id) = found_id {
      let cmd = format!("c --id {} move --workspace {}", id, entry.workspace);
      client.send(&cmd).await?;
      let _ = client.client_response(&cmd).await;
      println!("Moved existing managed window to {}", entry.workspace);
      continue;
    }

    // Not currently managed; enumerate native manageable windows and try to match.
    if let Ok(native_windows) = Platform::manageable_windows() {
      let mut native_match: Option<wm_platform::NativeWindow> = None;

      for nw in native_windows {
        if managed_handles.contains(&nw.handle) {
          continue;
        }

        let proc = nw.process_name().ok();
        let title = nw.title().ok();

        if entry.process_name.as_ref().map(|p| Some(p) == proc.as_ref()).unwrap_or(false)
          || entry.title.as_ref().map(|t| title.as_ref().map(|s| s.contains(t)).unwrap_or(false)).unwrap_or(false)
        {
          native_match = Some(nw);
          break;
        }
      }

      if let Some(nw) = native_match {
        // Focus the native window to let WM manage it.
        let _ = nw.set_foreground();
        // Wait briefly for WM to pick it up.
        sleep(Duration::from_millis(300)).await;

        // Re-query WM windows.
        client.send("q windows").await?;
        let resp2 = client.client_response("q windows").await.context("No response after refocus.")?;
        let windows2 = match resp2.data {
          Some(ClientResponseData::Windows(wd)) => wd.windows,
          _ => vec![],
        };

        // Find by handle.
        for c in &windows2 {
          if let ContainerDto::Window(wdto) = c {
            if wdto.handle == nw.handle {
              let cmd = format!("c --id {} move --workspace {}", wdto.id, entry.workspace);
              client.send(&cmd).await?;
              let _ = client.client_response(&cmd).await;
              println!("Focused+moved '{}' to {}", wdto.title, entry.workspace);
              break;
            }
          }
        }
      } else {
        println!("No native match found for entry: process={:?} title={:?}", entry.process_name, entry.title);
      }
    } else {
      println!("Failed to enumerate native windows.");
    }
  }

  Ok(())
}
