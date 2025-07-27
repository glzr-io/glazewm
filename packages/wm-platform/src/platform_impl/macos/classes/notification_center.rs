use objc2::{
  define_class, msg_send, rc::Retained, runtime::AnyObject, sel,
  AnyThread, DefinedClass,
};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::{
  ns_string, NSNotification, NSNotificationCenter, NSNotificationName,
  NSObject,
};
use tokio::sync::mpsc;

use crate::PlatformEvent;

#[repr(C)]
pub(crate) struct NotificationHandlerIvars {
  events_tx: mpsc::UnboundedSender<PlatformEvent>,
}

define_class! {
  // Safety:
  // - The superclass `NSObject` does not have any subclassing requirements.
  // - `NotificationHandler` does not implement `Drop`.
  #[unsafe(super(NSObject))]
  #[ivars = Box<NotificationHandlerIvars>]
  pub(crate) struct NotificationHandler;

  // Safety: Each of these method signatures must match their invocations.
  impl NotificationHandler {
    #[unsafe(method(onEvent:))]
    fn on_event(&self, notif: &NSNotification) {
      tracing::info!("Received notification: {notif:#?}");
      self.emit_event(PlatformEvent::DisplaySettingsChanged);
    }

    // #[unsafe(method(emitEvent:))]
    // fn emit_event(&self, event: PlatformEvent) {
    //   if let Err(err) = self.events_tx.send(event) {
    //     tracing::warn!("Failed to send event: {err}");
    //   }
    // }
  }
}

impl NotificationHandler {
  pub fn new(
    events_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> Retained<Self> {
    let instance = Self::alloc()
      .set_ivars(Box::new(NotificationHandlerIvars { events_tx }));

    // Safety: The signature of `NSObject`'s `init` method is correct.
    unsafe { msg_send![super(instance), init] }
  }

  fn emit_event(&self, event: PlatformEvent) {
    // TODO: Add switch statement to handle different event types.
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
    notification_name: &NSNotificationName,
    observer: &AnyObject,
    object: Option<&AnyObject>,
  ) {
    println!("Adding observer for {notification_name}");
    self.inner.addObserver_selector_name_object(
      observer,
      sel!(onEvent:),
      Some(notification_name),
      object,
    );
  }

  pub unsafe fn remove_observer(
    &mut self,
    notification_name: &NSNotificationName,
    observer: &AnyObject,
    object: Option<&AnyObject>,
  ) {
    self.inner.removeObserver_name_object(
      observer,
      Some(notification_name),
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
