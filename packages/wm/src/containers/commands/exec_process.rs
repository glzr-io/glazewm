use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::process::Command;

pub fn exec_process(process_name: &str, args: &[String]) -> Result<()> {
  // Expand user profile environment variable
  let user_profile = env::var("USERPROFILE").context(
    "Failed to get environment variable for user profile directory",
  )?;

  let re = Regex::new(r"(?i)%userprofile%").unwrap();

  let args: Vec<String> = args
    .iter()
    .map(|arg| re.replace_all(arg, &user_profile).into_owned())
    .collect();

  let process_name =
    re.replace_all(&process_name, &user_profile).into_owned();

  // Start external process
  let output = Command::new(process_name)
    .args(args) // pass the args vector directly
    .current_dir(env::var("USERPROFILE").context(
      "Failed to get environment variable for user profile directory",
    )?)
    .output()
    .context("Failed to execute command")?;

  if !output.status.success() {
    error!("Error: {}", String::from_utf8_lossy(&output.stderr));
    return Err(anyhow::anyhow!("Process execution failed"));
  }

  Ok(())
}
