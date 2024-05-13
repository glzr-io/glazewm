use std::path::Path;

use anyhow::Context;
use tracing::info;
use windows::{
  core::PCWSTR,
  Win32::UI::{
    Shell::{
      ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
      SHELLEXECUTEINFOW,
    },
    WindowsAndMessaging::SW_NORMAL,
  },
};

pub fn shell_exec(command: &str) -> anyhow::Result<()> {
  let (program, args) = parse_command(command)?;
  info!("Parsed command program: '{}', args: '{}'.", program, args);

  let home_dir = home::home_dir()
    .context("Unable to get home directory.")?
    .to_str()
    .context("Invalid home directory.")?
    .to_owned();

  // Inlining the wide variables within the `SHELLEXECUTEINFOW` struct
  // causes issues where the pointer is dropped while `ShellExecuteExW` is
  // using it. This is likely a `windows-rs` bug, and we can avoid it by
  // keeping separate variables for the wide strings.
  let program_wide = to_wide(&program);
  let args_wide = to_wide(&args);
  let home_dir_wide = to_wide(&home_dir);

  // Using the built-in `Command::new` function in Rust launches the
  // program as a subprocess. This prevents Windows from cleaning up
  // handles held by our process (e.g. the IPC server port) until the
  // subprocess exits.
  let mut exec_info = SHELLEXECUTEINFOW {
    cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
    lpFile: PCWSTR(program_wide.as_ptr()),
    lpParameters: PCWSTR(args_wide.as_ptr()),
    lpDirectory: PCWSTR(home_dir_wide.as_ptr()),
    nShow: SW_NORMAL.0 as _,
    fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
    ..Default::default()
  };

  unsafe { ShellExecuteExW(&mut exec_info) }?;
  info!("Command executed successfully: {}.", command);

  Ok(())
}

/// Parses a command string into a program name/path and arguments. The
/// command string may contain environment variables wrapped in `%`
/// characters.
///
/// If the command string is a path, a file extension is required.
///
/// Returns a tuple containing the program name/path and arguments.
///
/// # Examples
///
/// ```
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
fn parse_command(command: &str) -> anyhow::Result<(String, String)> {
  // TODO: Expand environment variables in the command string.
  let expanded_command = command;

  let command_parts: Vec<&str> =
    expanded_command.trim().split_whitespace().collect();

  if command.starts_with("\"") {
    // Find the closing double quote.
    let (closing_index, _) =
      command.match_indices('"').nth(2).with_context(|| {
        format!("Command doesn't have an ending `\"`: '{}'.", command)
      })?;

    return Ok((
      command[1..closing_index].to_string(),
      command[closing_index + 1..].trim().to_string(),
    ));
  }

  // The first part is the program name if it doesn't contain a slash or
  // backslash.
  if let Some(first_part) = command_parts.first() {
    if !first_part.contains(&['/', '\\'][..]) {
      let args = command_parts[1..].join(" ");
      return Ok((first_part.to_string(), args));
    }
  }

  let mut cumulative_path = Vec::new();

  for (part_index, &part) in command_parts.iter().enumerate() {
    cumulative_path.push(part);

    if Path::new(&cumulative_path.join(" ")).is_file() {
      return Ok((
        cumulative_path.join(" "),
        command_parts[part_index + 1..].join(" "),
      ));
    }
  }

  anyhow::bail!("Program path is not valid for command '{}'.", command)
}

fn to_wide(string: &str) -> Vec<u16> {
  string.encode_utf16().chain(Some(0)).collect()
}
