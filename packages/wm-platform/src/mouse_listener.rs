use tokio::sync::mpsc;

use crate::{
  platform_event::MouseEvent, platform_impl, Dispatcher, MouseButton,
  MouseEventNotification,
};

/// Enabled mouse event kinds for configuring the listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MouseEventType {
  Move,
  LeftClickDown,
  LeftClickUp,
  RightClickDown,
  RightClickUp,
}

/// A listener for system-wide mouse events.
pub struct MouseListener {
  /// Receiver for outgoing mouse events.
  event_rx: mpsc::UnboundedReceiver<MouseEvent>,

  /// Sender for outgoing mouse events.
  event_tx: mpsc::UnboundedSender<MouseEvent>,

  /// Underlying mouse hook used to listen for mouse events.
  hook: Option<platform_impl::MouseHook>,

  /// Currently enabled mouse event kinds.
  enabled_events: Vec<MouseEventType>,

  dispatcher: Dispatcher,
}

impl MouseListener {
  /// Creates a new mouse listener with the specified enabled events.
  pub fn new(
    enabled_events: Vec<MouseEventType>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mouse_hook = Self::create_mouse_hook(
      &enabled_events,
      event_tx.clone(),
      dispatcher,
    )?;

    Ok(Self {
      event_rx,
      event_tx,
      hook: Some(mouse_hook),
      dispatcher: dispatcher.clone(),
      enabled_events,
    })
  }

  /// Returns the next mouse event from the listener.
  ///
  /// This will block until a mouse event is available.
  pub async fn next_event(&mut self) -> Option<MouseEvent> {
    self.event_rx.recv().await
  }

  /// Enables or disables the underlying mouse hook.
  pub fn enable(&mut self, enabled: bool) {
    if let Some(hook) = &mut self.hook {
      hook.enable(enabled);
    }
  }

  /// Updates the set of enabled mouse events at runtime.
  ///
  /// This will terminate the existing hook and create a new one with the
  /// new enabled events.
  pub fn set_enabled_events(
    &mut self,
    enabled_events: Vec<MouseEventType>,
  ) -> crate::Result<()> {
    // Dispose the existing hook (if any).
    if let Some(mut hook) = self.hook.take() {
      let _ = hook.terminate();
    }

    let updated_hook = Self::create_mouse_hook(
      &enabled_events,
      self.event_tx.clone(),
      &self.dispatcher,
    )?;

    self.enabled_events = enabled_events;
    self.hook = Some(updated_hook);
    Ok(())
  }

  /// Creates and starts the mouse hook with the given callback.
  fn create_mouse_hook(
    enabled_events: &[MouseEventType],
    event_tx: mpsc::UnboundedSender<MouseEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<platform_impl::MouseHook> {
    platform_impl::MouseHook::new(
      enabled_events,
      move |notification: MouseEventNotification| {
        let event_type = notification.0.event_type();

        match event_type {
          MouseEventType::Move => {
            let _ = event_tx.send(MouseEvent::MouseMove {
              position: notification.0.position(),
              pressed_buttons: notification.0.pressed_buttons(),
              notification,
            });
          }
          MouseEventType::LeftClickDown
          | MouseEventType::RightClickDown => {
            let _ = event_tx.send(MouseEvent::MouseButtonDown {
              position: notification.0.position(),
              button: if event_type == MouseEventType::LeftClickDown {
                MouseButton::Left
              } else {
                MouseButton::Right
              },
              pressed_buttons: notification.0.pressed_buttons(),
              notification,
            });
          }
          MouseEventType::LeftClickUp | MouseEventType::RightClickUp => {
            let _ = event_tx.send(MouseEvent::MouseButtonUp {
              position: notification.0.position(),
              button: if event_type == MouseEventType::LeftClickUp {
                MouseButton::Left
              } else {
                MouseButton::Right
              },
              pressed_buttons: notification.0.pressed_buttons(),
              notification,
            });
          }
        }
      },
      dispatcher,
    )
  }
}
