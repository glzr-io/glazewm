/// Utility macro that logs a warning and returns early if the given
/// expression is an error.
#[macro_export]
macro_rules! try_warn {
  ($expr:expr) => {
    match $expr {
      Ok(val) => val,
      Err(err) => {
        tracing::warn!("Operation failed: {:?}", err);
        return Ok(());
      }
    }
  };
}
