use wm::ipc_client::IpcClient;
use wm::user_config::UserConfig;

#[tokio::main]
async fn main() {
  let config = UserConfig::new(None).await.unwrap();
  match IpcClient::connect().await {
    Ok(mut client) => {
      loop {
        client.send("Watcher process | PING").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      }
    },
    Err(e) => {
      eprintln!("Failed to connect to IPC server: {:?}", e);
      loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      }
    }
  }

  println!("config: {:?}", config);
}
