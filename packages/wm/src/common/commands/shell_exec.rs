use tracing::info;

use crate::common::platform::Platform;

pub fn shell_exec(command: &str) -> anyhow::Result<()> {
  let (program, args) = Platform::parse_command(command)?;
  info!("Parsed command program: '{}', args: '{}'.", program, args);

  Platform::run_command(&program, &args)?;
  info!("Command executed successfully: {}.", command);

  Ok(())
}
