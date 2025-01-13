use tracing::info;
use wm_platform::Platform;

pub fn shell_exec(command: &str, hide_window: bool) -> anyhow::Result<()> {
  let (program, args) = Platform::parse_command(command)?;
  info!("Parsed command program: '{}', args: '{}'.", program, args);

  Platform::run_command(&program, &args, hide_window).map_err(|err| {
    anyhow::anyhow!(format!(
      "Failed to execute '{command}'.\n\nError: {err}"
    ))
  })?;

  Ok(())
}
