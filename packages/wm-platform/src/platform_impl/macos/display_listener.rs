use std::{
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

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
    if let Some(observer) = self.observer.take() {
      observer.with(|observer| {
        NotificationCenter::workspace_center().remove_observer(observer);
        NotificationCenter::default_center().remove_observer(observer);
        NotificationCenter::distributed_center().remove_observer(observer);
      })?;
    }
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
    let mut distributed_center = NotificationCenter::distributed_center();
    // Unlock can occur without a system wake or a screen configuration
    // change.
    // SAFETY: The observer is unregistered from every center in
    // `terminate` before it is dropped.
    unsafe {
      distributed_center.add_observer(
        NotificationName::ScreenUnlocked,
        &observer,
        None,
      );
    }

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
      for name in [
        NotificationName::WorkspaceDidWake,
        NotificationName::WorkspaceSessionDidBecomeActive,
        NotificationName::WorkspaceScreensDidWake,
      ] {
        workspace_center.add_observer(name, &observer, None);
      }
    }

    std::thread::spawn(move || {
      // Duration to suppress display change events after wake. macOS fires
      // several notifications after waking from sleep, and displays can
      // take 1-2 seconds to be reported as online.
      const WAKE_COALESCE_DURATION: Duration = Duration::from_secs(5);

      let generation = Arc::new(AtomicU64::new(0));
      let mut is_asleep = false;
      let mut wake_time: Option<Instant> = None;

      // Loop exits when the sender is dropped in `Self::terminate`.
      while let Some(event) = events_rx.blocking_recv() {
        match event {
          NotificationEvent::WorkspaceWillSleep => {
            is_asleep = true;
            generation.fetch_add(1, Ordering::Relaxed);
          }
          NotificationEvent::WorkspaceDidWake => {
            is_asleep = false;
            wake_time = Some(Instant::now());

            // Send a single display change event after the coalesce
            // duration to pick up any changes that occurred during wake.
            let event_tx = event_tx.clone();
            let current_generation =
              generation.fetch_add(1, Ordering::Relaxed) + 1;
            let generation = generation.clone();
            std::thread::spawn(move || {
              std::thread::sleep(WAKE_COALESCE_DURATION);

              if generation.load(Ordering::Relaxed) != current_generation {
                return;
              }
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

      generation.fetch_add(1, Ordering::Relaxed);
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
