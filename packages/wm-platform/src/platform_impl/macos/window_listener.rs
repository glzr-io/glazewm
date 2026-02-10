use std::collections::HashMap;

use objc2_app_kit::NSWorkspace;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    self,
    classes::{
      NotificationCenter, NotificationEvent, NotificationName,
      NotificationObserver,
    },
    Application, ApplicationObserver, ProcessId,
  },
  Dispatcher, WindowEvent,
};

/// macOS-specific window event notification.
#[derive(Clone, Debug)]
pub struct WindowEventNotificationInner {
  /// Name of the notification (e.g. `AXWindowMoved`).
  pub name: String,

  /// Pointer to the `AXUIElement` that triggered the notification.
  pub ax_element_ptr: *mut std::ffi::c_void,
}

unsafe impl Send for WindowEventNotificationInner {}

/// macOS-specific implementation of [`WindowListener`].
#[derive(Debug)]
pub(crate) struct WindowListener;

impl WindowListener {
  /// macOS-specific implementation of [`WindowListener::new`].
  pub(crate) fn new(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    dispatcher
      .dispatch_sync(|| Self::init(events_tx, dispatcher.clone()))??;

    Ok(Self)
  }

  fn init(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: Dispatcher,
  ) -> crate::Result<()> {
    let (observer, events_rx) = NotificationObserver::new();

    let workspace = unsafe { NSWorkspace::sharedWorkspace() };

    let mut workspace_center = NotificationCenter::workspace_center();

    for notification in [
      NotificationName::WorkspaceActiveSpaceDidChange,
      NotificationName::WorkspaceDidLaunchApplication,
      NotificationName::WorkspaceDidActivateApplication,
      NotificationName::WorkspaceDidDeactivateApplication,
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

    // Create observers for all running applications.
    let app_observers = running_apps
      .into_iter()
      .filter_map(|app| {
        Self::create_app_observer(&app, events_tx.clone()).ok()
      })
      .collect::<Vec<_>>();

    tracing::info!(
      "Registered observers for {} existing applications.",
      app_observers.len()
    );

    std::thread::spawn(move || {
      Self::listen_workspace_events(
        app_observers,
        events_rx,
        &events_tx,
        &dispatcher,
      );
    });

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(observer);
    std::mem::forget(workspace_center);

    Ok(())
  }

  fn listen_workspace_events(
    app_observers: Vec<ApplicationObserver>,
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

    // TODO: Need to abort this loop when the window listener is
    // terminated.
    while let Some(event) = events_rx.blocking_recv() {
      tracing::debug!("Received workspace event: {event:?}");

      match event {
        NotificationEvent::WorkspaceDidLaunchApplication(running_app) => {
          let events_tx = events_tx.clone();

          let Ok(Ok(app_observer)) = dispatcher.dispatch_sync(|| {
            let app = Application::new(running_app, dispatcher.clone());
            if !app.should_observe() {
              return Err(crate::Error::Platform(format!(
                "Skipped observer registration for PID {} (should ignore).",
                app.pid,
              )));
            }

            ApplicationObserver::new(&app, events_tx.clone(), false)
          }) else {
            continue;
          };

          if app_observers.contains_key(&app_observer.pid) {
            tracing::debug!(
              "Observer already exists for PID {}.",
              app_observer.pid
            );
            continue;
          }

          app_observers.insert(app_observer.pid, app_observer);
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

            observer.emit_all_windows_destroyed();
          }
        }
        NotificationEvent::WorkspaceDidActivateApplication(
          running_app,
        ) => {
          let Ok(Ok(Some(focused_window))) =
            dispatcher.dispatch_sync(|| {
              let app = Application::new(running_app, dispatcher.clone());
              app.focused_window()
            })
          else {
            continue;
          };

          let _ = events_tx.send(WindowEvent::Focused {
            window: focused_window,
            notification: crate::WindowEventNotification(None),
          });
        }
        NotificationEvent::WorkspaceDidHideApplication(running_app) => {
          if let Some(app_observer) =
            app_observers.get(unsafe { &running_app.processIdentifier() })
          {
            app_observer.emit_all_windows_hidden();
          }
        }
        NotificationEvent::WorkspaceDidUnhideApplication(running_app) => {
          if let Some(app_observer) =
            app_observers.get(unsafe { &running_app.processIdentifier() })
          {
            app_observer.emit_all_windows_shown();
          }
        }
        _ => {}
      }
    }
  }

  fn create_app_observer(
    app: &Application,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
  ) -> crate::Result<ApplicationObserver> {
    if !app.should_observe() {
      return Err(crate::Error::Platform(format!(
        "Skipped observer registration for PID {} (should ignore).",
        app.pid,
      )));
    }

    let app_observer_res = ApplicationObserver::new(app, events_tx, true);

    if let Err(err) = &app_observer_res {
      tracing::debug!(
        "Skipped observer registration for PID {}: {}",
        app.pid,
        err
      );
    }

    app_observer_res
  }

  /// macOS-specific implementation of [`WindowListener::terminate`].
  pub(crate) fn terminate(&mut self) {
    // TODO: Implement this.
    todo!()
  }
}
