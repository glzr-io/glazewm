use objc2::{
  define_class, msg_send, rc::Retained, runtime::AnyObject, sel,
  AnyThread, DefinedClass,
};
use objc2_app_kit::{
  NSApplicationDidChangeScreenParametersNotification,
  NSRunningApplication, NSWorkspace,
  NSWorkspaceActiveSpaceDidChangeNotification,
  NSWorkspaceDidActivateApplicationNotification,
  NSWorkspaceDidDeactivateApplicationNotification,
  NSWorkspaceDidLaunchApplicationNotification,
  NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_foundation::{
  ns_string, NSNotification, NSNotificationCenter, NSNotificationName,
  NSObject, NSString,
};
use tokio::sync::mpsc;

use crate::PlatformEvent;

#[derive(Debug)]
pub(crate) enum NotificationName {
  WorkspaceActiveSpaceDidChange,
  WorkspaceDidActivateApplication,
  WorkspaceDidDeactivateApplication,
  WorkspaceDidLaunchApplication,
  WorkspaceDidTerminateApplication,
  ApplicationDidChangeScreenParameters,
}

impl From<&NSNotificationName> for NotificationName {
  fn from(name: &NSNotificationName) -> Self {
    if name == unsafe { NSWorkspaceDidLaunchApplicationNotification } {
      Self::WorkspaceDidLaunchApplication
    } else if name
      == unsafe { NSWorkspaceDidActivateApplicationNotification }
    {
      Self::WorkspaceDidActivateApplication
    } else if name
      == unsafe { NSWorkspaceDidDeactivateApplicationNotification }
    {
      Self::WorkspaceDidDeactivateApplication
    } else if name
      == unsafe { NSWorkspaceDidTerminateApplicationNotification }
    {
      Self::WorkspaceDidTerminateApplication
    } else if name
      == unsafe { NSWorkspaceActiveSpaceDidChangeNotification }
    {
      Self::WorkspaceActiveSpaceDidChange
    } else if name
      == unsafe { NSApplicationDidChangeScreenParametersNotification }
    {
      Self::ApplicationDidChangeScreenParameters
    } else {
      panic!("Unknown notification name: {name}");
    }
  }
}

impl From<NotificationName> for &NSString {
  fn from(name: NotificationName) -> Self {
    match name {
      NotificationName::WorkspaceActiveSpaceDidChange => unsafe {
        NSWorkspaceActiveSpaceDidChangeNotification
      },
      NotificationName::WorkspaceDidActivateApplication => unsafe {
        NSWorkspaceDidActivateApplicationNotification
      },
      NotificationName::WorkspaceDidDeactivateApplication => unsafe {
        NSWorkspaceDidDeactivateApplicationNotification
      },
      NotificationName::WorkspaceDidLaunchApplication => unsafe {
        NSWorkspaceDidLaunchApplicationNotification
      },
      NotificationName::WorkspaceDidTerminateApplication => unsafe {
        NSWorkspaceDidTerminateApplicationNotification
      },
      NotificationName::ApplicationDidChangeScreenParameters => unsafe {
        NSApplicationDidChangeScreenParametersNotification
      },
    }
  }
}

#[repr(C)]
pub(crate) struct NotificationObserverIvars {
  events_tx: mpsc::UnboundedSender<PlatformEvent>,
}

define_class! {
  // Safety:
  // - The superclass `NSObject` does not have any subclassing requirements.
  // - `NotificationObserver` does not implement `Drop`.
  #[unsafe(super(NSObject))]
  #[ivars = Box<NotificationObserverIvars>]
  pub(crate) struct NotificationObserver;

  // Safety: Each of these method signatures must match their invocations.
  impl NotificationObserver {
    #[unsafe(method(onEvent:))]
    fn on_event(&self, notif: &NSNotification) {
      self.handle_event(notif);
    }
  }
}

impl NotificationObserver {
  pub fn new(
    events_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> Retained<Self> {
    let instance = Self::alloc()
      .set_ivars(Box::new(NotificationObserverIvars { events_tx }));

    // Safety: The signature of `NSObject`'s `init` method is correct.
    unsafe { msg_send![super(instance), init] }
  }

  fn handle_event(&self, notif: &NSNotification) {
    tracing::info!("Received notification: {notif:#?}");

    // TODO: Properly handle the events.
    match NotificationName::from(unsafe { &*notif.name() }) {
      NotificationName::WorkspaceActiveSpaceDidChange => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
      NotificationName::WorkspaceDidActivateApplication => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
      NotificationName::WorkspaceDidDeactivateApplication => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
      NotificationName::WorkspaceDidLaunchApplication => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
      NotificationName::WorkspaceDidTerminateApplication => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
      NotificationName::ApplicationDidChangeScreenParameters => {
        self.emit_event(PlatformEvent::DisplaySettingsChanged);
      }
    }
  }

  fn emit_event(&self, event: PlatformEvent) {
    if let Err(err) = self.ivars().events_tx.send(event) {
      tracing::warn!("Failed to send event: {err}");
    }
  }
}

#[derive(Debug)]
pub(crate) struct NotificationCenter {
  inner: Retained<NSNotificationCenter>,
}

impl NotificationCenter {
  pub fn workspace_center() -> Self {
    let center =
      unsafe { NSWorkspace::sharedWorkspace().notificationCenter() };

    Self { inner: center }
  }

  pub fn default_center() -> Self {
    let center = unsafe { NSNotificationCenter::defaultCenter() };

    Self { inner: center }
  }

  pub unsafe fn add_observer(
    &mut self,
    notification_name: NotificationName,
    observer: &NotificationObserver,
    object: Option<&AnyObject>,
  ) {
    tracing::info!("Adding observer for {notification_name:?}.");

    self.inner.addObserver_selector_name_object(
      observer,
      sel!(onEvent:),
      Some(notification_name.into()),
      object,
    );
  }

  pub unsafe fn remove_observer(
    &mut self,
    notification_name: NotificationName,
    observer: &NotificationObserver,
    object: Option<&AnyObject>,
  ) {
    tracing::info!("Removing observer for {notification_name:?}.");

    self.inner.removeObserver_name_object(
      observer,
      Some(notification_name.into()),
      object,
    );
  }
}

pub unsafe fn get_app_from_notification(
  notification: &NSNotification,
) -> Option<Retained<NSRunningApplication>> {
  let user_info = notification.userInfo()?;
  let bundle_id_str = ns_string!("NSWorkspaceApplicationKey");
  let app = user_info.objectForKey(bundle_id_str);
  app.map(|app| Retained::<AnyObject>::cast(app))
}
