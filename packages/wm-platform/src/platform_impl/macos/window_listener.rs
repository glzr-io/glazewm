use std::collections::HashMap;

use objc2_app_kit::{NSApplication, NSWorkspace};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    self,
    classes::{
      NotificationCenter, NotificationEvent, NotificationName,
      NotificationObserver,
    },
    Application, ApplicationObserver,
  },
  Dispatcher, WindowEvent,
};

#[derive(Debug)]
pub struct WindowListener {
  pub event_rx: mpsc::UnboundedReceiver<WindowEvent>,
}

impl WindowListener {
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    let (events_tx, event_rx) = mpsc::unbounded_channel();

    dispatcher
      .clone()
      .dispatch_sync(|| Self::init(events_tx, dispatcher))??;

    Ok(Self { event_rx })
  }

  fn init(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: Dispatcher,
  ) -> crate::Result<()> {
    let (observer, events_rx) = NotificationObserver::new();

    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let shared_app =
      NSApplication::sharedApplication(MainThreadMarker::new().unwrap());

    let mut workspace_center = NotificationCenter::workspace_center();
    let mut default_center = NotificationCenter::default_center();

    for notification in [
      NotificationName::WorkspaceActiveSpaceDidChange,
      NotificationName::WorkspaceDidLaunchApplication,
      NotificationName::WorkspaceDidActivateApplication,
      NotificationName::WorkspaceDidDeactivateApplication,
      NotificationName::WorkspaceDidTerminateApplication,
    ] {
      unsafe {
        workspace_center.add_observer(
          notification,
          &observer,
          Some(&workspace),
        );
      }
    }

    unsafe {
      default_center.add_observer(
        NotificationName::ApplicationDidChangeScreenParameters,
        &observer,
        Some(&shared_app),
      );
    }

    let running_apps = platform_impl::all_applications(&dispatcher)?;

    // Create observers for all running applications.
    let app_observers = running_apps
      .into_iter()
      .filter_map(|app| {
        Self::create_app_observer(&app, events_tx.clone(), &dispatcher)
          .ok()
      })
      .collect::<Vec<_>>();

    tracing::info!(
      "Registered observers for {} existing applications.",
      app_observers.len()
    );

    std::thread::spawn(move || {
      Self::listen_events(app_observers, events_rx, events_tx, dispatcher);
    });

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(observer);
    std::mem::forget(workspace_center);
    std::mem::forget(default_center);

    Ok(())
  }

  fn listen_events(
    app_observers: Vec<ApplicationObserver>,
    mut events_rx: mpsc::UnboundedReceiver<NotificationEvent>,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: Dispatcher,
  ) {
    // Track window observers for each application by PID.
    let mut app_observers: HashMap<i32, ApplicationObserver> =
      app_observers
        .into_iter()
        .map(|observer| (observer.pid, observer))
        .collect();

    while let Some(event) = events_rx.blocking_recv() {
      tracing::debug!("Received workspace event: {event:?}");

      match event {
        NotificationEvent::WorkspaceDidLaunchApplication(running_app) => {
          let pid = unsafe { running_app.processIdentifier() };

          if app_observers.contains_key(&pid) {
            tracing::debug!("Observer already exists for PID {}.", pid);
            continue;
          }

          match ApplicationObserver::new(
            pid,
            events_tx.clone(),
            &dispatcher,
          ) {
            Ok(observer) => {
              tracing::info!(
                "Registered window observer for PID: {}",
                pid
              );
              app_observers.insert(pid, observer);
            }
            Err(err) => {
              tracing::warn!(
                "Failed to register window observer for PID {}: {}",
                pid,
                err
              );
            }
          }
        }
        NotificationEvent::WorkspaceDidTerminateApplication(
          running_app,
        ) => {
          let pid = unsafe { running_app.processIdentifier() };

          if let Some(observer) = app_observers.remove(&pid) {
            tracing::info!(
              "Removed window observer for terminated PID: {}",
              pid
            );
            drop(observer); // Triggers cleanup in Drop implementation
          }
        }
        _ => {}
      }
    }
  }

  fn create_app_observer(
    app: &Application,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<ApplicationObserver> {
    let app_observer_res =
      ApplicationObserver::new(app.pid, events_tx, dispatcher);

    if let Err(err) = &app_observer_res {
      tracing::debug!(
        "Skipped observer registration for PID {}: {}",
        app.pid,
        err
      );
    }

    app_observer_res
  }

  /// Returns the next event from the `WindowListener`.
  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    self.event_rx.recv().await
  }
}
