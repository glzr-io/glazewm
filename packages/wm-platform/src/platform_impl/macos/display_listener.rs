use std::time::{Duration, Instant};

use objc2::rc::Retained;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    NotificationCenter, NotificationEvent, NotificationName,
    NotificationObserver,
  },
  Dispatcher, ThreadBound,
};

/// Platform-specific implementation of [`DisplayListener`].
pub(crate) struct DisplayListener {
  /// Notification observer bound to the main thread.
  observer: Option<ThreadBound<Retained<NotificationObserver>>>,
}

impl DisplayListener {
  /// Creates an instance of `DisplayListener`.
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<()>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let dispatcher_clone = dispatcher.clone();
    let observer = dispatcher.dispatch_sync(move || {
      Self::add_observers(event_tx, dispatcher_clone)
    })?;

    Ok(Self {
      observer: Some(observer),
    })
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

  /// Registers notification observers on the main thread.
  fn add_observers(
    event_tx: mpsc::UnboundedSender<()>,
    dispatcher: Dispatcher,
  ) -> ThreadBound<Retained<NotificationObserver>> {
    let (observer, mut events_rx) = NotificationObserver::new();
    let mut default_center = NotificationCenter::default_center();
    let mut workspace_center = NotificationCenter::workspace_center();

    // Add observer which will fire when displays are connected and
    // disconnected, resolution changes, or arrangement changes.
    unsafe {
      default_center.add_observer(
        NotificationName::ApplicationDidChangeScreenParameters,
        &observer,
        None,
      );
    }

    // Add observers for system sleep and wake events.
    unsafe {
      workspace_center.add_observer(
        NotificationName::WorkspaceWillSleep,
        &observer,
        None,
      );
      workspace_center.add_observer(
        NotificationName::WorkspaceDidWake,
        &observer,
        None,
      );
    }

    std::thread::spawn(move || {
      // Duration to suppress display change events after wake. macOS fires
      // several notifications after waking from sleep, and displays can
      // take 1-2 seconds to be reported as online.
      const WAKE_COALESCE_DURATION: Duration = Duration::from_secs(5);

      let mut is_asleep = false;
      let mut wake_time: Option<Instant> = None;

      // Loop exits when the sender is dropped in `Self::terminate`.
      while let Some(event) = events_rx.blocking_recv() {
        match event {
          NotificationEvent::WorkspaceWillSleep => {
            is_asleep = true;
          }
          NotificationEvent::WorkspaceDidWake => {
            is_asleep = false;
            wake_time = Some(Instant::now());

            // Send a single display change event after the coalesce
            // duration to pick up any changes that occurred during wake.
            let event_tx = event_tx.clone();
            std::thread::spawn(move || {
              std::thread::sleep(WAKE_COALESCE_DURATION);

              if let Err(err) = event_tx.send(()) {
                tracing::warn!(
                  "Failed to send display change event: {}",
                  err
                );
              }
            });
          }
          NotificationEvent::ApplicationDidChangeScreenParameters => {
            // Ignore display change events while asleep or within the
            // coalesce duration after wake.
            if is_asleep
              || wake_time
                .is_some_and(|t| t.elapsed() < WAKE_COALESCE_DURATION)
            {
              continue;
            }

            // Coalesce duration has passed; clear it.
            if wake_time.is_some() {
              wake_time = None;
            }

            if let Err(err) = event_tx.send(()) {
              tracing::warn!(
                "Failed to send display change event: {}",
                err
              );
              break;
            }
          }
          _ => {}
        }
      }

      tracing::debug!("Display listener thread exited.");
    });

    ThreadBound::new(observer, dispatcher)
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
