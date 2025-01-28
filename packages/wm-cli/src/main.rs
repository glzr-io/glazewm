use std::{env, process::Command};

use anyhow::Context;
use wm_cli::start;
use wm_common::AppCommand;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
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
