use accessibility::{
  AXAttribute, AXUIElement, AXUIElementActions, AXUIElementAttributes,
};
use accessibility_sys::{
  kAXApplicationActivatedNotification,
  kAXApplicationDeactivatedNotification, kAXErrorCannotComplete,
  kAXMainWindowChangedNotification, kAXStandardWindowSubrole,
  kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
  kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
  kAXWindowMiniaturizedNotification, kAXWindowMovedNotification,
  kAXWindowResizedNotification, kAXWindowRole,
};
use objc2_app_kit::{NSApplication, NSWorkspace};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    classes::{
      NotificationCenter, NotificationEvent, NotificationName,
      NotificationObserver,
    },
    EventLoopDispatcher,
  },
  PlatformEvent,
};

#[derive(Debug)]
pub struct WindowListener {
  pub event_rx: mpsc::UnboundedReceiver<PlatformEvent>,
}

impl WindowListener {
  pub fn new(dispatcher: &EventLoopDispatcher) -> anyhow::Result<Self> {
    let (events_tx, event_rx) = mpsc::unbounded_channel();

    let dispatcher_clone = dispatcher.clone();
    dispatcher.dispatch_sync(|| {
      Self::add_observers(events_tx, dispatcher_clone)
    })?;

    Ok(Self { event_rx })
  }

  fn add_observers(
    events_tx: mpsc::UnboundedSender<PlatformEvent>,
    dispatcher: EventLoopDispatcher,
  ) {
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

    std::thread::spawn(move || Self::listen(events_rx, dispatcher));

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(observer);
    std::mem::forget(workspace_center);
    std::mem::forget(default_center);
  }

  fn listen(
    mut events_rx: mpsc::UnboundedReceiver<NotificationEvent>,
    dispatcher: EventLoopDispatcher,
  ) {
    while let Some(event) = events_rx.blocking_recv() {
      tracing::info!("Received event: {event:?}");

      match event {
        NotificationEvent::WorkspaceDidLaunchApplication => {
          tracing::info!("Workspace launched application.");

          // TODO: Register window event listeners for the new application.
        }
        _ => {}
      }
    }
  }

  /// Returns the next event from the `WindowListener`.
  pub async fn next_event(&mut self) -> Option<PlatformEvent> {
    self.event_rx.recv().await
  }
}
