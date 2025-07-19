pub struct EventLoopDispatcher {
  message_window_handle: crate::WindowHandle,
  thread_id: u32,
}

impl EventLoopDispatcher {
  pub fn dispatch<F>(&self, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    // Move the current dispatch implementation here
  }
}
