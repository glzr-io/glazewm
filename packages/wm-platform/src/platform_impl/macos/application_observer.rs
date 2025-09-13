use accessibility_sys::{
  kAXFocusedWindowChangedNotification, kAXMainWindowChangedNotification,
  kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
  kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
  kAXWindowMiniaturizedNotification, kAXWindowMovedNotification,
  kAXWindowResizedNotification,
};
use dispatch2::MainThreadBound;
use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource, CFString,
};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    AXObserverAddNotification, AXObserverCreate,
    AXObserverGetRunLoopSource, AXObserverRef,
    AXUIElementCreateApplication, AXUIElementExt, AXUIElementRef,
    CFStringRef, NativeWindow, ProcessId,
  },
  Dispatcher, WindowEvent,
};

const AX_APP_NOTIFICATIONS: &[&str] = &[
  kAXFocusedWindowChangedNotification,
  kAXWindowCreatedNotification,
];

const AX_WINDOW_NOTIFICATIONS: &[&str] = &[
  kAXTitleChangedNotification,
  kAXUIElementDestroyedNotification,
  kAXWindowMovedNotification,
  kAXWindowResizedNotification,
  kAXWindowDeminiaturizedNotification,
  kAXWindowMiniaturizedNotification,
];

/// Context data passed to the window event callback.
struct WindowEventContext {
  events_tx: mpsc::UnboundedSender<WindowEvent>,
  dispatcher: Dispatcher,
  pid: ProcessId,
}

/// Represents an accessibility observer for a specific application.
#[derive(Debug)]
pub(crate) struct AppWindowObserver {
  pub(crate) pid: ProcessId,
  observer: AXObserverRef,
  app_element: AXUIElementRef,
  runloop_source: CFRetained<CFRunLoopSource>,
  context: *mut WindowEventContext,
}

impl AppWindowObserver {
  pub fn new(
    pid: ProcessId,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let app_element = unsafe { AXUIElementCreateApplication(pid) };
    if app_element.is_null() {
      return Err(crate::Error::Platform(format!(
        "Failed to create AXUIElement for PID {pid}"
      )));
    }

    let mut observer: AXObserverRef = std::ptr::null_mut();
    let result = unsafe {
      AXObserverCreate(pid, Self::window_event_callback, &mut observer)
    };

    if result != AXError::Success {
      return Err(crate::Error::Accessibility(
        "AXObserverCreate".to_string(),
        result.0,
      ));
    }

    let context = Box::into_raw(Box::new(WindowEventContext {
      events_tx,
      dispatcher: dispatcher.clone(),
      pid,
    }));

    let runloop_source = unsafe {
      let source = AXObserverGetRunLoopSource(observer);
      CFRetained::retain(std::ptr::NonNull::new_unchecked(source.cast()))
    };

    unsafe {
      let runloop =
        CFRunLoop::current().ok_or(crate::Error::EventLoopStopped)?;
      runloop.add_source(Some(&runloop_source), kCFRunLoopDefaultMode);
    }

    // Register for all window notifications
    Self::register_notifications(observer, app_element, context)?;

    Ok(Self {
      observer,
      pid,
      app_element,
      runloop_source,
      context,
    })
  }

  fn register_notifications(
    observer: AXObserverRef,
    app_element: AXUIElementRef,
    context: *mut WindowEventContext,
  ) -> crate::Result<()> {
    let notifications = [
      kAXWindowCreatedNotification,
      kAXUIElementDestroyedNotification,
      kAXWindowMovedNotification,
      kAXWindowResizedNotification,
      kAXWindowMiniaturizedNotification,
      kAXWindowDeminiaturizedNotification,
      kAXTitleChangedNotification,
      kAXMainWindowChangedNotification,
    ];

    for notification in &notifications {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        let result = AXObserverAddNotification(
          observer,
          app_element,
          &notification_cfstr,
          context.cast::<std::ffi::c_void>(),
        );

        if result != AXError::Success {
          tracing::warn!(
            "Failed to add notification {} for PID {}: {:?}",
            notification,
            (*context).pid,
            result
          );
        }
      }
    }
    Ok(())
  }

  /// Callback function for accessibility window events.
  unsafe extern "C" fn window_event_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut std::ffi::c_void,
  ) {
    if context.is_null() {
      tracing::error!("Window event callback received null context");
      return;
    }

    let context = &*(context as *const WindowEventContext);
    let cf_string: CFRetained<CFString> = unsafe {
      CFRetained::retain(std::ptr::NonNull::new_unchecked(
        notification.cast_mut(),
      ))
    };

    let notification_str = cf_string.to_string();

    // Retain the element for safe access.
    let ax_element = match AXUIElement::from_ref(element) {
      Ok(el) => el,
      Err(err) => {
        tracing::error!(
          "Failed to retain AXUIElement in callback: {}",
          err
        );
        return;
      }
    };

    tracing::info!(
      "Received window event: {} for PID: {}",
      notification_str,
      context.pid
    );

    let ax_element_ref =
      MainThreadBound::new(ax_element, MainThreadMarker::new().unwrap());

    // TODO: Extract proper CGWindowID from AX element instead of using 0
    let window =
      NativeWindow::new(0, context.dispatcher.clone(), ax_element_ref);

    let window_event = match notification_str.as_str() {
      kAXWindowCreatedNotification => {
        tracing::info!("Window created for PID: {}", context.pid);
        Some(WindowEvent::Show(window.into()))
      }
      kAXUIElementDestroyedNotification => {
        tracing::info!("Window destroyed for PID: {}", context.pid);
        Some(WindowEvent::Destroy(window.id()))
      }
      kAXWindowMovedNotification | kAXWindowResizedNotification => {
        tracing::debug!("Window moved/resized for PID: {}", context.pid);
        Some(WindowEvent::LocationChange(window.into()))
      }
      kAXWindowMiniaturizedNotification => {
        tracing::info!("Window minimized for PID: {}", context.pid);
        Some(WindowEvent::Minimize(window.into()))
      }
      kAXWindowDeminiaturizedNotification => {
        tracing::info!("Window deminimized for PID: {}", context.pid);
        Some(WindowEvent::MinimizeEnd(window.into()))
      }
      kAXTitleChangedNotification => {
        tracing::debug!("Window title changed for PID: {}", context.pid);
        Some(WindowEvent::TitleChange(window.into()))
      }
      kAXMainWindowChangedNotification => {
        tracing::debug!("Main window changed for PID: {}", context.pid);
        Some(WindowEvent::Focus(window.into()))
      }
      _ => {
        tracing::debug!(
          "Unhandled window notification: {} for PID: {}",
          notification_str,
          context.pid
        );
        None
      }
    };

    if let Some(event) = window_event {
      if let Err(err) = context.events_tx.send(event) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          context.pid,
          err
        );
      }
    }
  }
}

impl Drop for AppWindowObserver {
  fn drop(&mut self) {
    tracing::debug!("Cleaning up AppWindowObserver for PID {}", self.pid);

    // Remove all notifications.
    let notifications = [
      kAXWindowCreatedNotification,
      kAXUIElementDestroyedNotification,
      kAXWindowMovedNotification,
      kAXWindowResizedNotification,
      kAXWindowMiniaturizedNotification,
      kAXWindowDeminiaturizedNotification,
      kAXTitleChangedNotification,
      kAXMainWindowChangedNotification,
    ];

    for notification in &notifications {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        crate::platform_impl::AXObserverRemoveNotification(
          self.observer,
          self.app_element,
          &notification_cfstr,
        );
      }
    }

    // Remove from run loop.
    unsafe {
      if let Some(runloop) = CFRunLoop::current() {
        runloop.remove_source(
          Some(&self.runloop_source),
          kCFRunLoopDefaultMode,
        );
      }
    }

    // Clean up context.
    if !self.context.is_null() {
      let _context = unsafe { Box::from_raw(self.context) };
    }

    tracing::debug!(
      "AppWindowObserver cleanup completed for PID {}",
      self.pid
    );
  }
}
