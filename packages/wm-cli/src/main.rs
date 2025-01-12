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
      let main_path = env::current_exe()?
        .parent()
        .and_then(|path| {
          path
            .join(if cfg!(debug_assertions) {
              "glazewm.exe"
            } else {
              "../glazewm.exe"
            })
            .to_str()
            .map(|path| path.to_string())
        })
        .context("Failed to resolve path to the main executable.")?;

      Command::new("cmd")
        .args(["/S", "/C", "start", "", &main_path])
        .spawn()
        .context("Failed to start main executable.")?;

      Ok(())
    }
    _ => start(args).await,
  }
}
