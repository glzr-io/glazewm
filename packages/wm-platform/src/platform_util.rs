pub struct PlatformUtil;

impl PlatformUtil {
  #[cfg(target_os = "macos")]
  pub fn is_main_thread() -> anyhow::Result<bool> {
    let main_thread = MainThreadMarker::new().ok_or_else(|| {
      Error::Anyhow(anyhow::anyhow!("Not on main thread"))
    })?;
  }
}
