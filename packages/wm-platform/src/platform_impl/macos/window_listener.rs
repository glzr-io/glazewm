use objc2_app_kit::{NSApplication, NSWorkspace};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    classes::{
      NotificationCenter, NotificationName, NotificationObserver,
    },
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

    dispatcher.dispatch_sync(|| Self::add_observers(events_tx))?;

    Ok(Self { event_rx })
  }

  fn add_observers(events_tx: mpsc::UnboundedSender<PlatformEvent>) {
    let observer = NotificationObserver::new(events_tx);

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

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(observer);
    std::mem::forget(workspace_center);
    std::mem::forget(default_center);
  }
}
