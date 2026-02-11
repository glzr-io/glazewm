use objc2::rc::Retained;
use objc2_app_kit::NSApplication;
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;
use tracing::warn;

use crate::{
  platform_impl::classes::{
    NotificationCenter, NotificationEvent, NotificationName,
    NotificationObserver,
  },
  Dispatcher, ThreadBound,
};

/// Listens for display configuration changes on macOS.
pub struct DisplayListener {
  events_rx: mpsc::UnboundedReceiver<NotificationEvent>,
  /// Notification observer bound to the main thread.
  observer: Option<ThreadBound<Retained<NotificationObserver>>>,
}

impl DisplayListener {
  /// Creates a new `DisplayListener` and registers for display change
  /// notifications.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let dispatcher_clone = dispatcher.clone();
    let (observer, events_rx) = dispatcher
      .dispatch_sync(move || Self::add_observer(dispatcher_clone))??;

    Ok(Self {
      events_rx,
      observer: Some(observer),
    })
  }

  /// Registers the notification observer on the main thread.
  fn add_observer(
    dispatcher: Dispatcher,
  ) -> crate::Result<(
    ThreadBound<Retained<NotificationObserver>>,
    mpsc::UnboundedReceiver<NotificationEvent>,
  )> {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    let (observer, events_rx) = NotificationObserver::new();
    let shared_app = NSApplication::sharedApplication(mtm);

    let mut default_center = NotificationCenter::default_center();

    // Add observer which will fire when displays are connected and
    // disconnected, resolution changes, or arrangement changes.
    unsafe {
      default_center.add_observer(
        NotificationName::ApplicationDidChangeScreenParameters,
        &observer,
        Some(&shared_app),
      );
    }

    let observer = ThreadBound::new(observer, dispatcher);

    Ok((observer, events_rx))
  }

  /// Returns when the next display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<()> {
    self.events_rx.recv().await.map(|_| ())
  }

  /// Deregisters the display change observer from `NSNotificationCenter`.
  pub fn terminate(&mut self) -> crate::Result<()> {
    let Some(observer) = self.observer.take() else {
      return Ok(());
    };

    observer.with(|observer| {
      let mtm =
        MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

      let shared_app = NSApplication::sharedApplication(mtm);
      let mut default_center = NotificationCenter::default_center();

      unsafe {
        default_center.remove_observer(
          NotificationName::ApplicationDidChangeScreenParameters,
          observer,
          Some(&shared_app),
        );
      }

      crate::Result::Ok(())
    })?
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      warn!("Failed to terminate display listener: {}", err);
    }
  }
}
