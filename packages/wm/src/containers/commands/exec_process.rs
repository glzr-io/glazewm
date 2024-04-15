use std::process::Command;
use std::env;
use anyhow::{Result, Context};

pub fn exec_process(process_name: &str, args: &[String]) -> Result<()> {
    
    let output = Command::new(process_name)
        .args(args) // pass the args vector directly
        .current_dir(env::var("USERPROFILE")
            .context("Failed to get environment variable for user profile directory")?)
        .output()
        .context("Failed to execute command")?;

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Process execution failed"));
    }

    Ok(())
}

