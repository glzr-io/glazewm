use wm::user_config::UserConfig;

#[tokio::main]
async fn main() {
  let config = UserConfig::read(None).await.unwrap();
  let config = config.lock().await;
  println!("config: {:?}", *config);
}
