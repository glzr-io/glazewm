use std::path::Path;

#[cfg(target_os = "windows")]
use anyhow::Context;
#[cfg(target_os = "macos")]
use shell_util::{CommandOptions, Shell};
#[cfg(target_os = "windows")]
use wm_platform::DispatcherExtWindows;

use crate::wm_state::WmState;

pub fn shell_exec(
  command: &str,
  // LINT: `hide_window` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  hide_window: bool,
  state: &WmState,
) -> anyhow::Result<()> {
  let (program, args) = parse_command(command, state)?;
  tracing::info!(
    "Parsed command program: '{}', args: '{}'.",
    program,
    args
  );

  // NOTE: The standard library's `Command::new` is not used because it
  // launches the program as a subprocess. This prevents cleanup of handles
  // held by our process (e.g. the IPC server port) until the subprocess
  // exits.
  let result = {
    #[cfg(target_os = "macos")]
    {
      Shell::spawn(
        &program,
        args.split_whitespace(),
        &CommandOptions::default(),
      )
    }
    #[cfg(target_os = "windows")]
    {
      let home_dir =
        home::home_dir().context("Unable to get home directory.")?;

      // TODO: Use `Shell::spawn` instead. `ShellExecuteExW` is still used
      // to be able to launch programs from the App Paths registry
      // (`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths`), like
      // `chrome` without it being in $PATH.
      state.dispatcher.shell_execute_ex(
        &program,
        &args,
        &home_dir,
        hide_window,
      )
    }
  };

  result.map_err(|err| {
    anyhow::anyhow!(
      "Shell exec failed for '{command}'. Make sure the program exists and is \
      accessible from your shell. Error: {err}",
    )
  })?;

  Ok(())
}

/// Parses a command string into a program name/path and arguments. This
/// also expands any environment variables found in the command string if
/// they are wrapped in `%` characters. If the command string is a path,
/// a file extension is required.
///
/// This is similar to the `SHEvaluateSystemCommandTemplate` Win32
/// function. It also parses program name/path and arguments, but can't
/// handle `/` as file path delimiters and it errors for certain programs
/// (e.g. `code`).
///
/// Returns a tuple containing the program name/path and arguments.
///
/// # Examples
///
/// ```no_run
/// let (prog, args) = parse_command("code .")?;
/// assert_eq!(prog, "code");
/// assert_eq!(args, ".");
///
/// let (prog, args) = parse_command(
///   r#"C:\Program Files\Git\git-bash --cd=C:\Users\larsb\.glaze-wm"#,
/// )?;
/// assert_eq!(prog, r#"C:\Program Files\Git\git-bash"#);
/// assert_eq!(args, r#"--cd=C:\Users\larsb\.glaze-wm"#);
/// ```
fn parse_command(
  command: &str,
  // LINT: `state` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  state: &WmState,
) -> anyhow::Result<(String, String)> {
  // Expand environment variables in the command string.
  let expanded_command = {
    #[cfg(target_os = "windows")]
    {
      state.dispatcher.expand_env_strings(command)?
    }
    #[cfg(target_os = "macos")]
    {
      // TODO: Expand env variables on macOS.
      command.to_string()
    }
  };

  let command_parts =
    expanded_command.split_whitespace().collect::<Vec<_>>();

  // If the command starts with double quotes, then the program name/path
  // is wrapped in double quotes (e.g. `"C:\path\to\app.exe" --flag`).
  if expanded_command.starts_with('"') {
    // Find the closing double quote.
    let (closing_index, _) =
      expanded_command.match_indices('"').nth(2).ok_or_else(|| {
        anyhow::anyhow!(
          "Shell exec failed for '{command}': command doesn't have an ending `\"`."
        )
      })?;

    return Ok((
      expanded_command[1..closing_index].to_string(),
      expanded_command[closing_index + 1..].trim().to_string(),
    ));
  }

  // The first part is the program name if it doesn't contain a slash or
  // backslash.
  if let Some(first_part) = command_parts.first() {
    if !first_part.contains(&['/', '\\'][..]) {
      let args = command_parts[1..].join(" ");
      return Ok(((*first_part).to_string(), args));
    }
  }

  let mut cumulative_path = Vec::new();

  // Lastly, iterate over the command until a valid file path is found.
  for (part_index, &part) in command_parts.iter().enumerate() {
    cumulative_path.push(part);

    if Path::new(&cumulative_path.join(" ")).is_file() {
      return Ok((
        cumulative_path.join(" "),
        command_parts[part_index + 1..].join(" "),
      ));
    }
  }

  anyhow::bail!(
    "Shell exec failed for '{command}': program path is not valid."
  )
}
