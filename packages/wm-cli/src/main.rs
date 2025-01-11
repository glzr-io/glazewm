use wm_cli::start;
use wm_common::AppCommand;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  match app_command {
    AppCommand::Start {
      config_path,
      verbosity,
    } => {
      // TODO: Launch the executable.
      Ok(())
    }
    _ => start(args).await,
  }
}
