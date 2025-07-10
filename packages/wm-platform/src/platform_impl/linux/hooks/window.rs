use crate::WindowEvent;

#[derive(Debug)]
pub struct WindowEventHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<WindowEvent>,
}

#[derive(Debug)]
pub struct EventThreadWindowEventHook {
  tx: tokio::sync::mpsc::UnboundedSender<WindowEvent>,
}
