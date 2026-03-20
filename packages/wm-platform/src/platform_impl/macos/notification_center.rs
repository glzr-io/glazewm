use objc2::{
  define_class, msg_send, rc::Retained, runtime::AnyObject, sel,
  AnyThread, DefinedClass,
};
use objc2_app_kit::{
  NSApplicationDidChangeScreenParametersNotification,
  NSRunningApplication, NSWorkspace,
  NSWorkspaceActiveSpaceDidChangeNotification,
  NSWorkspaceDidActivateApplicationNotification,
  NSWorkspaceDidHideApplicationNotification,
  NSWorkspaceDidLaunchApplicationNotification,
  NSWorkspaceDidTerminateApplicationNotification,
  NSWorkspaceDidUnhideApplicationNotification,
  NSWorkspaceDidWakeNotification, NSWorkspaceWillSleepNotification,
};
use objc2_foundation::{
  ns_string, NSNotification, NSNotificationCenter, NSNotificationName,
  NSObject, NSString,
};
use tokio::sync::mpsc;

/// Notification names for observing macOS workspace and screen events.
#[derive(Debug)]
pub(crate) enum NotificationName {
  WorkspaceActiveSpaceDidChange,
  WorkspaceDidActivateApplication,
  WorkspaceDidLaunchApplication,
  WorkspaceDidTerminateApplication,
  WorkspaceDidHideApplication,
  WorkspaceDidUnhideApplication,
  WorkspaceDidWake,
  WorkspaceWillSleep,
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
      == unsafe { NSWorkspaceDidTerminateApplicationNotification }
    {
      Self::WorkspaceDidTerminateApplication
    } else if name
      == unsafe { NSWorkspaceActiveSpaceDidChangeNotification }
    {
      Self::WorkspaceActiveSpaceDidChange
    } else if name == unsafe { NSWorkspaceDidHideApplicationNotification }
    {
      Self::WorkspaceDidHideApplication
    } else if name
      == unsafe { NSWorkspaceDidUnhideApplicationNotification }
    {
      Self::WorkspaceDidUnhideApplication
    } else if name == unsafe { NSWorkspaceDidWakeNotification } {
      Self::WorkspaceDidWake
    } else if name == unsafe { NSWorkspaceWillSleepNotification } {
      Self::WorkspaceWillSleep
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
      NotificationName::WorkspaceDidLaunchApplication => unsafe {
        NSWorkspaceDidLaunchApplicationNotification
      },
      NotificationName::WorkspaceDidTerminateApplication => unsafe {
        NSWorkspaceDidTerminateApplicationNotification
      },
      NotificationName::WorkspaceDidHideApplication => unsafe {
        NSWorkspaceDidHideApplicationNotification
      },
      NotificationName::WorkspaceDidUnhideApplication => unsafe {
        NSWorkspaceDidUnhideApplicationNotification
      },
      NotificationName::WorkspaceDidWake => unsafe {
        NSWorkspaceDidWakeNotification
      },
      NotificationName::WorkspaceWillSleep => unsafe {
        NSWorkspaceWillSleepNotification
      },
      NotificationName::ApplicationDidChangeScreenParameters => unsafe {
        NSApplicationDidChangeScreenParametersNotification
      },
    }
  }
}

/// Events received from macOS notification center observers.
#[derive(Debug)]
pub(crate) enum NotificationEvent {
  WorkspaceActiveSpaceDidChange,
  WorkspaceDidActivateApplication(Retained<NSRunningApplication>),
  WorkspaceDidLaunchApplication(Retained<NSRunningApplication>),
  WorkspaceDidTerminateApplication(Retained<NSRunningApplication>),
  WorkspaceDidHideApplication(Retained<NSRunningApplication>),
  WorkspaceDidUnhideApplication(Retained<NSRunningApplication>),
  WorkspaceWillSleep,
  WorkspaceDidWake,
  ApplicationDidChangeScreenParameters,
}

/// Instance variables for `NotificationObserver`.
#[repr(C)]
pub(crate) struct NotificationObserverIvars {
  events_tx: mpsc::UnboundedSender<NotificationEvent>,
}

define_class! {
  // SAFETY:
  // - The superclass `NSObject` does not have any subclassing requirements.
  // - `NotificationObserver` does not implement `Drop`.
  #[unsafe(super(NSObject))]
  #[ivars = Box<NotificationObserverIvars>]
  pub(crate) struct NotificationObserver;

  // SAFETY: Each of these method signatures must match their invocations.
  impl NotificationObserver {
    #[unsafe(method(onEvent:))]
    fn on_event(&self, notif: &NSNotification) {
      self.handle_event(notif);
    }
  }
}

impl NotificationObserver {
  pub fn new(
  ) -> (Retained<Self>, mpsc::UnboundedReceiver<NotificationEvent>) {
    let (events_tx, events_rx) = mpsc::unbounded_channel();

    let instance = Self::alloc()
      .set_ivars(Box::new(NotificationObserverIvars { events_tx }));

    // SAFETY: The signature of `NSObject`'s `init` method is correct.
    (unsafe { msg_send![super(instance), init] }, events_rx)
  }

  fn handle_event(&self, notif: &NSNotification) {
    tracing::debug!("Received notification: {notif:#?}");

    match NotificationName::from(&*notif.name()) {
      NotificationName::WorkspaceActiveSpaceDidChange => {
        self.emit_event(NotificationEvent::WorkspaceActiveSpaceDidChange);
      }
      NotificationName::WorkspaceDidActivateApplication => {
        if let Some(app) = unsafe { app_from_notification(notif) } {
          self.emit_event(
            NotificationEvent::WorkspaceDidActivateApplication(app),
          );
        } else {
          tracing::warn!(
            "Failed to extract application from activate notification"
          );
        }
      }
      NotificationName::WorkspaceDidLaunchApplication => {
        if let Some(app) = unsafe { app_from_notification(notif) } {
          self.emit_event(
            NotificationEvent::WorkspaceDidLaunchApplication(app),
          );
        } else {
          tracing::warn!(
            "Failed to extract application from launch notification"
          );
        }
      }
      NotificationName::WorkspaceDidTerminateApplication => {
        if let Some(app) = unsafe { app_from_notification(notif) } {
          self.emit_event(
            NotificationEvent::WorkspaceDidTerminateApplication(app),
          );
        } else {
          tracing::warn!(
            "Failed to extract application from terminate notification"
          );
        }
      }
      NotificationName::WorkspaceDidHideApplication => {
        if let Some(app) = unsafe { app_from_notification(notif) } {
          self.emit_event(NotificationEvent::WorkspaceDidHideApplication(
            app,
          ));
        }
      }
      NotificationName::WorkspaceDidUnhideApplication => {
        if let Some(app) = unsafe { app_from_notification(notif) } {
          self.emit_event(
            NotificationEvent::WorkspaceDidUnhideApplication(app),
          );
        }
      }
      NotificationName::WorkspaceDidWake => {
        self.emit_event(NotificationEvent::WorkspaceDidWake);
      }
      NotificationName::WorkspaceWillSleep => {
        self.emit_event(NotificationEvent::WorkspaceWillSleep);
      }
      NotificationName::ApplicationDidChangeScreenParameters => {
        self.emit_event(
          NotificationEvent::ApplicationDidChangeScreenParameters,
        );
      }
    }
  }

  fn emit_event(&self, event: NotificationEvent) {
    if let Err(err) = self.ivars().events_tx.send(event) {
      tracing::warn!("Failed to send event: {err}");
    }
  }
}

/// Wrapper around `NSNotificationCenter` for registering event observers.
#[derive(Debug)]
pub(crate) struct NotificationCenter {
  inner: Retained<NSNotificationCenter>,
}

impl NotificationCenter {
  pub fn workspace_center() -> Self {
    let center = NSWorkspace::sharedWorkspace().notificationCenter();

    Self { inner: center }
  }

  pub fn default_center() -> Self {
    let center = NSNotificationCenter::defaultCenter();

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
}

pub unsafe fn app_from_notification(
  notification: &NSNotification,
) -> Option<Retained<NSRunningApplication>> {
  notification
    .userInfo()?
    .objectForKey(ns_string!("NSWorkspaceApplicationKey"))
    .map(|app| Retained::<AnyObject>::cast_unchecked(app))
}
