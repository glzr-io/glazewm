use tracing::{error, info};

use wm_platform::Platform;

pub fn shell_exec(command: &str, hide_window: bool) -> anyhow::Result<()> {
  let res =
    Platform::parse_command(command).and_then(|(program, args)| {
      info!("Parsed command program: '{}', args: '{}'.", program, args);

      Platform::run_command(&program, &args, hide_window)
    });

  match res {
    Ok(_) => {
      info!("Command executed successfully: {}.", command);
    }
    Err(err) => {
      let err_message =
        format!("Failed to execute '{}'.\n\nError: {}", command, err);

      error!(err_message);
      Platform::show_error_dialog("Non-fatal error", &err_message);
    }
  }

  Ok(())
}
