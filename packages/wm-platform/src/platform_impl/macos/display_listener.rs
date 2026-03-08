use objc2::rc::Retained;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    NotificationCenter, NotificationName, NotificationObserver,
  },
  Dispatcher, ThreadBound,
};

/// Platform-specific implementation of [`DisplayListener`].
pub(crate) struct DisplayListener {
  /// Notification observer bound to the main thread.
  observer: Option<ThreadBound<Retained<NotificationObserver>>>,
}

impl DisplayListener {
  /// Implements [`DisplayListener::new`].
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<()>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let dispatcher_clone = dispatcher.clone();
    let observer = dispatcher.dispatch_sync(move || {
      Self::add_observer(event_tx, dispatcher_clone)
    })?;

    Ok(Self {
      observer: Some(observer),
    })
  }

  /// Registers the notification observer on the main thread.
  fn add_observer(
    event_tx: mpsc::UnboundedSender<()>,
    dispatcher: Dispatcher,
  ) -> ThreadBound<Retained<NotificationObserver>> {
    let (observer, mut events_rx) = NotificationObserver::new();
    let mut default_center = NotificationCenter::default_center();

    // Add observer which will fire when displays are connected and
    // disconnected, resolution changes, or arrangement changes.
    unsafe {
      default_center.add_observer(
        NotificationName::ApplicationDidChangeScreenParameters,
        &observer,
        None,
      );
    }

    std::thread::spawn(move || {
      // Loop exits when the sender is dropped in `Self::terminate`.
      while events_rx.blocking_recv().is_some() {
        if let Err(err) = event_tx.send(()) {
          tracing::warn!("Failed to send display change event: {}", err);
          break;
        }
      }

      tracing::debug!("Display listener thread exited.");
    });

    ThreadBound::new(observer, dispatcher)
  }

  /// Implements [`DisplayListener::terminate`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    // On macOS 10.11+, observer subscriptions are cleaned up automatically
    // without calling `removeObserver`.
    // Ref: https://developer.apple.com/documentation/foundation/notificationcenter/removeobserver(_:name:object:)
    //
    // Dropping the `NotificationObserver` also drops its channel sender,
    // causing the listener thread to exit.
    self.observer.take();
    Ok(())
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
