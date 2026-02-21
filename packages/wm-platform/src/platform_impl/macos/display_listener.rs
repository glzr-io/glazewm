use objc2::rc::Retained;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    NotificationCenter, NotificationName, NotificationObserver,
  },
  Dispatcher, ThreadBound,
};

/// macOS-specific implementation of [`DisplayListener`].
pub(crate) struct DisplayListener {
  /// Notification observer bound to the main thread.
  observer: Option<ThreadBound<Retained<NotificationObserver>>>,
}

impl DisplayListener {
  /// macOS-specific implementation of [`DisplayListener::new`].
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
      while events_rx.blocking_recv().is_some() {
        if let Err(err) = event_tx.send(()) {
          tracing::warn!("Failed to send display change event: {}", err);
          break;
        }
      }
    });

    ThreadBound::new(observer, dispatcher)
  }

  /// macOS-specific implementation of [`DisplayListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    let Some(observer) = self.observer.take() else {
      return Ok(());
    };

    observer.with(|observer| {
      let mut default_center = NotificationCenter::default_center();

      unsafe {
        default_center.remove_observer(
          NotificationName::ApplicationDidChangeScreenParameters,
          observer,
          None,
        );
      }
    })
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      tracing::warn!("Failed to terminate display listener: {}", err);
    }
  }
}
