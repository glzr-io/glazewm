use std::env::ArgsOs;

use anyhow::Result;

pub const DEFAULT_IPC_ADDR: &'static str = "127.0.0.1:6123";

pub struct IpcClient {}

impl IpcClient {
  pub async fn connect() -> Result<()> {
    todo!()
  }

  pub async fn send_raw(_message: ArgsOs) -> Result<()> {
    todo!()
  }
}
