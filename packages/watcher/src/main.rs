use wm::user_config::UserConfig;

#[tokio::main]
async fn main() {
  let config = UserConfig::new(None).await.unwrap();
  println!("config: {:?}", config);
}
