use std::{collections::HashMap, time::Duration};

use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use tokio::{sync::mpsc, time::Instant};

use crate::{
  platform_impl::{
    self, Application, ApplicationObserver, NotificationCenter,
    NotificationEvent, NotificationName, NotificationObserver, ProcessId,
  },
  Dispatcher, ThreadBound, WindowEvent,
};

// A bounded retry budget covers apps whose Accessibility server is still
// starting. Keep retries in the listener so shutdown and termination
// cancel them without background tasks or callback pointers.
const OBSERVER_RETRY_DELAY: Duration = Duration::from_millis(250);
const MAX_OBSERVER_ATTEMPTS: u8 = 8;

#[derive(Debug)]
struct PendingObserver {
  app: Application,
  attempts: u8,
  deadline: Instant,
}

fn should_retry_observer(error: &crate::Error, attempts: u8) -> bool {
  matches!(error, crate::Error::Accessibility(_, code)
    if *code == objc2_application_services::AXError::CannotComplete.0)
    && attempts < MAX_OBSERVER_ATTEMPTS
}

enum ListenerWake<T> {
  Event(T),
  Retry,
  Closed,
}

async fn receive_or_retry<T>(
  receiver: &mut mpsc::UnboundedReceiver<T>,
  deadline: Option<Instant>,
) -> ListenerWake<T> {
  tokio::select! {
    // Termination and shutdown take precedence over an already due retry.
    biased;
    event = receiver.recv() => match event {
      Some(event) => ListenerWake::Event(event),
      None => ListenerWake::Closed,
    },
    () = async {
      if let Some(deadline) = deadline {
        tokio::time::sleep_until(deadline).await;
      } else {
        std::future::pending::<()>().await;
      }
    } => ListenerWake::Retry,
  }
}

/// Platform-specific implementation of [`WindowEventNotification`].
#[derive(Clone, Debug)]
pub struct WindowEventNotificationInner {
  /// Name of the notification (e.g. `AXWindowMoved`).
  pub name: String,

  /// Pointer to the `AXUIElement` that triggered the notification.
  pub ax_element_ptr: *mut std::ffi::c_void,
}

unsafe impl Send for WindowEventNotificationInner {}

/// Platform-specific implementation of [`WindowListener`].
#[derive(Debug)]
pub(crate) struct WindowListener {
  /// Workspace notification observer, bound to the main thread.
  observer: Option<ThreadBound<Retained<NotificationObserver>>>,
}

impl WindowListener {
  /// Implements [`WindowListener::new`].
  pub(crate) fn new(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let observer = dispatcher
      .dispatch_sync(|| Self::init(events_tx, dispatcher.clone()))??;

    Ok(Self {
      observer: Some(observer),
    })
  }

  /// Implements [`WindowListener::terminate`].
  pub(crate) fn terminate(&mut self) {
    // On macOS 10.11+, observer subscriptions are cleaned up automatically
    // without calling `removeObserver`.
    // Ref: https://developer.apple.com/documentation/foundation/notificationcenter/removeobserver(_:name:object:)
    //
    // Dropping the `NotificationObserver` also drops its channel sender,
    // causing the listener thread to exit.
    self.observer.take();
  }

  fn init(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: Dispatcher,
  ) -> crate::Result<ThreadBound<Retained<NotificationObserver>>> {
    let (observer, events_rx) = NotificationObserver::new();

    let workspace = NSWorkspace::sharedWorkspace();
    let mut workspace_center = NotificationCenter::workspace_center();

    for notification in [
      NotificationName::WorkspaceActiveSpaceDidChange,
      NotificationName::WorkspaceDidLaunchApplication,
      NotificationName::WorkspaceDidActivateApplication,
      NotificationName::WorkspaceDidTerminateApplication,
      NotificationName::WorkspaceDidHideApplication,
      NotificationName::WorkspaceDidUnhideApplication,
    ] {
      unsafe {
        workspace_center.add_observer(
          notification,
          &observer,
          Some(&workspace),
        );
      }
    }

    let running_apps = platform_impl::all_applications(&dispatcher)?;

    let mut app_observers = Vec::new();
    let mut pending = HashMap::new();
    for app in running_apps {
      if !app.should_observe() {
        continue;
      }
      match ApplicationObserver::new(&app, events_tx.clone(), true) {
        Ok(observer) => app_observers.push(observer),
        Err(error) => Self::schedule_retry(&mut pending, app, 1, &error),
      }
    }

    tracing::info!(
      "Registered observers for {} existing applications.",
      app_observers.len()
    );

    let dispatcher_clone = dispatcher.clone();
    std::thread::spawn(move || {
      let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
      {
        Ok(runtime) => runtime,
        Err(error) => {
          tracing::error!("Failed to start window listener: {error}");
          return;
        }
      };
      runtime.block_on(Self::listen_workspace_events(
        app_observers,
        pending,
        events_rx,
        &events_tx,
        &dispatcher_clone,
      ));
    });

    Ok(ThreadBound::new(observer, dispatcher))
  }

  async fn listen_workspace_events(
    app_observers: Vec<ApplicationObserver>,
    mut pending: HashMap<ProcessId, PendingObserver>,
    mut events_rx: mpsc::UnboundedReceiver<NotificationEvent>,
    events_tx: &mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) {
    // Track window observers for each application by PID.
    let mut app_observers: HashMap<ProcessId, ApplicationObserver> =
      app_observers
        .into_iter()
        .map(|observer| (observer.pid, observer))
        .collect();

    // Loop exits when the sender is dropped in `Self::terminate`.
    loop {
      let deadline = pending.values().map(|retry| retry.deadline).min();
      let event = match receive_or_retry(&mut events_rx, deadline).await {
        ListenerWake::Closed => break,
        ListenerWake::Event(event) => event,
        ListenerWake::Retry => {
          Self::retry_due_observers(
            &mut app_observers,
            &mut pending,
            events_tx,
            dispatcher,
          );
          continue;
        }
      };
      tracing::debug!("Received workspace event: {event:?}");

      match event {
        NotificationEvent::WorkspaceDidLaunchApplication(running_app) => {
          let pid = running_app.processIdentifier();
          if app_observers.contains_key(&pid) || pending.contains_key(&pid)
          {
            continue;
          }
          Self::register_application(
            running_app,
            &mut app_observers,
            &mut pending,
            events_tx,
            dispatcher,
          );
        }
        NotificationEvent::WorkspaceDidTerminateApplication(
          running_app,
        ) => {
          let pid = running_app.processIdentifier();
          pending.remove(&pid);

          if let Some(observer) = app_observers.remove(&pid) {
            tracing::info!(
              "Removed window observer for terminated PID: {}",
              pid
            );

            observer.emit_all_windows_destroyed();
          }
        }
        NotificationEvent::WorkspaceDidActivateApplication(
          running_app,
        ) => {
          let pid = running_app.processIdentifier();

          if app_observers.contains_key(&pid) {
            let Ok(Ok(Some(focused_window))) =
              dispatcher.dispatch_sync(|| {
                let app =
                  Application::new(running_app, dispatcher.clone());
                app.focused_window()
              })
            else {
              continue;
            };

            let _ = events_tx.send(WindowEvent::Focused {
              window: focused_window,
              notification: crate::WindowEventNotification(None),
            });
          } else if !pending.contains_key(&pid) {
            Self::register_application(
              running_app,
              &mut app_observers,
              &mut pending,
              events_tx,
              dispatcher,
            );
          }
        }
        NotificationEvent::WorkspaceDidHideApplication(running_app) => {
          if let Some(app_observer) =
            app_observers.get(&running_app.processIdentifier())
          {
            app_observer.emit_all_windows_hidden();
          }
        }
        NotificationEvent::WorkspaceDidUnhideApplication(running_app) => {
          if let Some(app_observer) =
            app_observers.get(&running_app.processIdentifier())
          {
            app_observer.emit_all_windows_shown();
          }
        }
        _ => {}
      }
    }

    tracing::debug!("Window listener thread exited.");
  }

  fn retry_due_observers(
    app_observers: &mut HashMap<ProcessId, ApplicationObserver>,
    pending: &mut HashMap<ProcessId, PendingObserver>,
    events_tx: &mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) {
    let due: Vec<_> = pending
      .iter()
      .filter(|(_, retry)| retry.deadline <= Instant::now())
      .map(|(pid, _)| *pid)
      .collect();
    for pid in due {
      let retry = pending.remove(&pid).unwrap();
      let result = dispatcher
        .dispatch_sync(|| {
          if retry.app.ns_app.isTerminated() {
            return Ok(None);
          }
          ApplicationObserver::new(&retry.app, events_tx.clone(), false)
            .map(Some)
        })
        .and_then(|result| result);
      match result {
        Ok(Some(observer)) => {
          tracing::info!(
            "Registered window observer for PID {pid} after retry."
          );
          app_observers.insert(pid, observer);
        }
        Ok(None) => {}
        Err(error) => Self::schedule_retry(
          pending,
          retry.app,
          retry.attempts + 1,
          &error,
        ),
      }
    }
  }

  fn register_application(
    running_app: Retained<objc2_app_kit::NSRunningApplication>,
    observers: &mut HashMap<ProcessId, ApplicationObserver>,
    pending: &mut HashMap<ProcessId, PendingObserver>,
    events_tx: &mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) {
    let result = dispatcher.dispatch_sync(|| {
      let app = Application::new(running_app, dispatcher.clone());
      if !app.should_observe() || app.ns_app.isTerminated() {
        return None;
      }
      let result =
        ApplicationObserver::new(&app, events_tx.clone(), false);
      Some((app, result))
    });
    match result {
      Ok(Some((app, Ok(observer)))) => {
        observers.insert(app.pid, observer);
      }
      Ok(Some((app, Err(error)))) => {
        Self::schedule_retry(pending, app, 1, &error);
      }
      Ok(None) => {}
      Err(error) => {
        tracing::warn!(
          "Failed to dispatch observer registration: {error}"
        );
      }
    }
  }

  fn schedule_retry(
    pending: &mut HashMap<ProcessId, PendingObserver>,
    app: Application,
    attempts: u8,
    error: &crate::Error,
  ) {
    let pid = app.pid;
    if should_retry_observer(error, attempts) {
      tracing::debug!("Retrying observer registration for PID {pid} after attempt {attempts}: {error}");
      pending.insert(
        pid,
        PendingObserver {
          app,
          attempts,
          deadline: Instant::now()
            + (OBSERVER_RETRY_DELAY * (1 << (attempts - 1)))
              .min(Duration::from_secs(2)),
        },
      );
    } else {
      tracing::warn!("Failed to register window observer for PID {pid} after {attempts} attempt(s): {error}");
    }
  }
}

impl Drop for WindowListener {
  fn drop(&mut self) {
    self.terminate();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn transient_registration_failure_has_a_bounded_budget() {
    let error = crate::Error::Accessibility("AXWindows".into(), -25204);
    for attempt in 1..MAX_OBSERVER_ATTEMPTS {
      assert!(should_retry_observer(&error, attempt));
    }
    assert!(!should_retry_observer(&error, MAX_OBSERVER_ATTEMPTS));
    for code in [-25202, -25205, -25211] {
      assert!(!should_retry_observer(
        &crate::Error::Accessibility("AXWindows".into(), code),
        1,
      ));
    }
    assert!(!should_retry_observer(&crate::Error::EventLoopStopped, 1));
  }

  #[test]
  fn retry_wakes_without_another_workspace_event() {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_time()
      .build()
      .unwrap();
    runtime.block_on(async {
      let (_sender, mut receiver) = mpsc::unbounded_channel::<()>();
      assert!(matches!(
        receive_or_retry(
          &mut receiver,
          Some(Instant::now() + Duration::from_millis(1)),
        )
        .await,
        ListenerWake::Retry
      ));
    });
  }

  #[test]
  fn shutdown_and_queued_events_precede_due_retries() {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_time()
      .build()
      .unwrap();
    runtime.block_on(async {
      let (sender, mut receiver) = mpsc::unbounded_channel();
      sender.send("terminated").unwrap();
      assert!(matches!(
        receive_or_retry(&mut receiver, Some(Instant::now())).await,
        ListenerWake::Event("terminated")
      ));
      drop(sender);
      assert!(matches!(
        receive_or_retry(&mut receiver, Some(Instant::now())).await,
        ListenerWake::Closed
      ));
      assert!(matches!(
        receive_or_retry(&mut receiver, None).await,
        ListenerWake::Closed
      ));
    });
  }
}
