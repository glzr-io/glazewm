use objc2_app_kit::{
  NSApplication, NSApplicationDidChangeScreenParametersNotification,
  NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
  NSWorkspaceDidActivateApplicationNotification,
  NSWorkspaceDidDeactivateApplicationNotification,
  NSWorkspaceDidLaunchApplicationNotification,
  NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    classes::{NotificationCenter, NotificationHandler},
    EventLoopDispatcher,
  },
  PlatformEvent,
};

pub struct WindowListener {
  pub event_rx: mpsc::UnboundedReceiver<PlatformEvent>,
}

impl WindowListener {
  pub fn new(dispatcher: &EventLoopDispatcher) -> anyhow::Result<Self> {
    let (events_tx, event_rx) = mpsc::unbounded_channel();

    dispatcher.run_sync(|| Self::add_observers(events_tx))?;

    Ok(Self { event_rx })
  }

  fn add_observers(events_tx: mpsc::UnboundedSender<PlatformEvent>) {
    let handler = NotificationHandler::new(events_tx);

    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let shared_app =
      NSApplication::sharedApplication(MainThreadMarker::new().unwrap());

    let workspace_notifications = unsafe {
      [
        NSWorkspaceActiveSpaceDidChangeNotification,
        NSWorkspaceDidLaunchApplicationNotification,
        NSWorkspaceDidActivateApplicationNotification,
        NSWorkspaceDidDeactivateApplicationNotification,
        NSWorkspaceDidTerminateApplicationNotification,
      ]
    };

    let app_notifications =
      unsafe { [NSApplicationDidChangeScreenParametersNotification] };

    let mut workspace_center = NotificationCenter::workspace_center();
    let mut default_center = NotificationCenter::default_center();

    for notification in workspace_notifications {
      unsafe {
        workspace_center.add_observer(
          notification,
          &handler,
          Some(&workspace),
        );
      }
    }

    for notification in app_notifications {
      unsafe {
        default_center.add_observer(
          notification,
          &handler,
          Some(&shared_app),
        );
      }
    }

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(handler);
    std::mem::forget(workspace_center);
    std::mem::forget(default_center);
  }
}
