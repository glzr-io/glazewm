use std::{
  sync::Mutex,
  time::{Duration, Instant},
};

use tokio::sync::mpsc;

use crate::{
  platform_event::{MouseEvent, PressedButtons},
  platform_impl, Dispatcher, MouseButton, Point, WindowId,
};

/// Available mouse events that [`MouseListener`] can listen for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MouseEventKind {
  Move,
  LeftButtonDown,
  LeftButtonUp,
  RightButtonDown,
  RightButtonUp,
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
  enabled_events: Vec<MouseEventKind>,

  dispatcher: Dispatcher,
}

impl MouseListener {
  /// Creates a new mouse listener with the specified enabled events.
  pub fn new(
    enabled_events: Vec<MouseEventKind>,
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
    enabled_events: Vec<MouseEventKind>,
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
    enabled_events: &[MouseEventKind],
    event_tx: mpsc::UnboundedSender<MouseEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<platform_impl::MouseHook> {
    // Timestamp of the last mouse move event emission.
    let last_move_emission = Mutex::new(None::<Instant>);

    platform_impl::MouseHook::new(
      enabled_events,
      move |event_kind: MouseEventKind,
            position: Point,
            pressed_buttons: PressedButtons,
            window_below_cursor: Option<WindowId>| {
        match event_kind {
          MouseEventKind::Move => {
            let mut last_move_emission = last_move_emission
              .lock()
              .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Throttle mouse move events so that there's a minimum of 50ms
            // between each emission. State change events (button down/up)
            // always get emitted.
            let has_elapsed_throttle =
              last_move_emission.is_none_or(|timestamp| {
                timestamp.elapsed() >= Duration::from_millis(50)
              });

            let should_emit = {
              #[cfg(target_os = "windows")]
              {
                has_elapsed_throttle
              }
              #[cfg(target_os = "macos")]
              {
                // TODO: This is a hack to let through mouse move events
                // when they contain a window ID. macOS sporadically
                // includes the window ID on mouse events.
                has_elapsed_throttle || window_below_cursor.is_some()
              }
            };

            if should_emit {
              let _ = event_tx.send(MouseEvent::Move {
                position,
                pressed_buttons,
                window_below_cursor,
              });

              *last_move_emission = Some(Instant::now());
            }
          }
          MouseEventKind::LeftButtonDown
          | MouseEventKind::RightButtonDown => {
            let _ = event_tx.send(MouseEvent::ButtonDown {
              position,
              button: if event_kind == MouseEventKind::LeftButtonDown {
                MouseButton::Left
              } else {
                MouseButton::Right
              },
              pressed_buttons,
            });
          }
          MouseEventKind::LeftButtonUp | MouseEventKind::RightButtonUp => {
            let _ = event_tx.send(MouseEvent::ButtonUp {
              position,
              button: if event_kind == MouseEventKind::LeftButtonUp {
                MouseButton::Left
              } else {
                MouseButton::Right
              },
              pressed_buttons,
            });
          }
        }
      },
      dispatcher,
    )
  }
}
