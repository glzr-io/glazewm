#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
  #[command(subcommand)]
  pub message: WmMessage,
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.message {
    WmMessage::Start { config_path } => {
      let wm_path = env::var_os("CARGO_BIN_FILE_WM")
        .context("Failed to resolve path to the WM process.")?;

      Command::new(wm_path)
    }
    _ => {
      let args = std::env::args_os();
      IpcClient::connect()
        .await
        .unwrap()
        .send_raw(args)
        .await
        .unwrap()
    }
  }
}
