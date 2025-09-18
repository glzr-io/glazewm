use std::{ptr::NonNull, sync::Arc};

use accessibility_sys::{
  kAXFocusedWindowChangedNotification, kAXMainWindowChangedNotification,
  kAXTitleChangedNotification, kAXUIElementDestroyedNotification,
  kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
  kAXWindowMiniaturizedNotification, kAXWindowMovedNotification,
  kAXWindowResizedNotification,
};
use dispatch2::MainThreadBound;
use objc2_application_services::{AXError, AXObserver, AXUIElement};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource, CFString,
};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{Application, NativeWindow, ProcessId},
  NativeWindowExtMacOs, WindowEvent, WindowId,
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

/// Context data passed to the application event callback.
#[derive(Debug)]
struct ApplicationEventContext {
  pid: ProcessId,
  events_tx: mpsc::UnboundedSender<WindowEvent>,
  app_windows: Vec<crate::NativeWindow>,
  observer: CFRetained<AXObserver>,
}

/// Represents an accessibility observer for a specific application.
#[derive(Debug)]
pub(crate) struct ApplicationObserver {
  pub(crate) pid: ProcessId,
  observer: CFRetained<AXObserver>,
  observer_source: CFRetained<CFRunLoopSource>,
  app_element: Arc<MainThreadBound<CFRetained<AXUIElement>>>,
  // context: Box<ApplicationEventContext>,
}

// TODO: Remove this.
unsafe impl Send for ApplicationObserver {}

impl ApplicationObserver {
  pub fn new(
    app: &Application,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
  ) -> crate::Result<Self> {
    // Creation of `AXUIElement` for an application does not fail even if
    // the PID is invalid. Instead, subsequent operations on the returned
    // `AXUIElement` will error.
    // let app_element = unsafe { AXUIElement::new_application(pid) };

    let observer = unsafe {
      let mut observer = std::ptr::null_mut();

      let result = AXObserver::create(
        app.pid,
        Some(Self::window_event_callback),
        // SAFETY: Stack address of `observer` is guaranteed to be
        // non-null.
        NonNull::new(&raw mut observer).unwrap(),
      );

      if result != AXError::Success {
        return Err(crate::Error::Accessibility(
          "AXObserverCreate".to_string(),
          result.0,
        ));
      }

      CFRetained::retain(NonNull::new(observer).ok_or_else(|| {
        crate::Error::InvalidPointer("AXObserver is null.".to_string())
      })?)
    };

    let app_windows = app.windows()?;
    let context = Box::into_raw(Box::new(ApplicationEventContext {
      pid: app.pid,
      events_tx: events_tx.clone(),
      app_windows: app_windows.clone(),
      observer: observer.clone(),
    }));

    let runloop =
      CFRunLoop::current().ok_or(crate::Error::EventLoopStopped)?;

    let observer_source = unsafe { observer.run_loop_source() };
    runloop.add_source(Some(&observer_source), unsafe {
      kCFRunLoopDefaultMode
    });

    // Register for all window notifications.
    // TODO: Remove from runloop if registration fails.
    Self::register_app_notifications(app, &observer, context)?;

    // Emit `WindowEvent::Show` for all existing windows.
    for window in app_windows {
      if let Err(err) =
        Self::register_window_notifications(&window, &observer, context)
      {
        tracing::warn!(
          "Failed to register window notifications for PID {}: {}",
          app.pid,
          err
        );
      }

      if let Err(err) = events_tx.send(WindowEvent::Show(window)) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          app.pid,
          err
        );
      }
    }

    Ok(Self {
      pid: app.pid,
      observer,
      observer_source,
      app_element: app.ax_element.clone(),
      // context,
    })
  }

  fn register_app_notifications(
    app: &Application,
    observer: &CFRetained<AXObserver>,
    context: *mut ApplicationEventContext,
  ) -> crate::Result<()> {
    let mtm = MainThreadMarker::new().unwrap();

    for notification in AX_APP_NOTIFICATIONS {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        let result = observer.add_notification(
          app.ax_element.get(mtm),
          &notification_cfstr,
          context.cast::<std::ffi::c_void>(),
        );

        if result != AXError::Success {
          return Err(crate::Error::Platform(format!(
            "Failed to add notification {} for PID {}: {:?}",
            notification, app.pid, result
          )));
        }
      }
    }

    Ok(())
  }

  fn register_window_notifications(
    window: &crate::NativeWindow,
    observer: &CFRetained<AXObserver>,
    context: *mut ApplicationEventContext,
  ) -> crate::Result<()> {
    let mtm = MainThreadMarker::new().unwrap();

    for notification in AX_WINDOW_NOTIFICATIONS {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        let result = observer.add_notification(
          window.ax_ui_element().get(mtm),
          &notification_cfstr,
          context.cast::<std::ffi::c_void>(),
        );

        if result != AXError::Success {
          return Err(crate::Error::Platform(format!(
            "Failed to add notification {} for window {}: {:?}",
            notification,
            window.id().0,
            result
          )));
        }
      }
    }

    Ok(())
  }

  /// Callback function for accessibility window events.
  unsafe extern "C-unwind" fn window_event_callback(
    _observer: NonNull<AXObserver>,
    element: NonNull<AXUIElement>,
    notification: NonNull<CFString>,
    context: *mut std::ffi::c_void,
  ) {
    if context.is_null() {
      tracing::error!("Window event callback received null context.");
      return;
    }

    let context = &mut *context.cast::<ApplicationEventContext>();
    let notification_str = notification.as_ref().to_string();
    let ax_element = unsafe { CFRetained::retain(element) };

    tracing::info!(
      "Received window event: {} for PID: {}",
      notification_str,
      context.pid
    );

    let mtm = MainThreadMarker::new().unwrap();
    let found_window = context
      .app_windows
      .iter()
      .find(|window| window.ax_ui_element().get(mtm) == &ax_element);

    let window_id = WindowId::from_window_element(&ax_element);
    let ax_element =
      MainThreadBound::new(ax_element, MainThreadMarker::new().unwrap());

    if let Some(window) = found_window {
      println!("Found window: {:?} {:?}", window.id(), window_id);
    } else {
      println!("Window not found: {:?}", window_id);
    }

    let window: crate::NativeWindow =
      NativeWindow::new(window_id, ax_element).into();

    let window_event = match notification_str.as_str() {
      kAXWindowCreatedNotification => {
        tracing::info!("Window created for PID: {}", context.pid);
        context.app_windows.push(window.clone());
        let observer = context.observer.clone();
        let _ =
          Self::register_window_notifications(&window, &observer, context);
        Some(WindowEvent::Show(window))
      }
      kAXUIElementDestroyedNotification => {
        tracing::info!("Window destroyed for PID: {}", context.pid);
        if let Some(window) = found_window {
          // TODO: Should remove the window from the list of windows.
          // context.app_windows.retain(|w| w.id() != window.id());
          Some(WindowEvent::Destroy(window.id()))
        } else {
          None
        }
      }
      kAXWindowMovedNotification | kAXWindowResizedNotification => {
        tracing::debug!("Window moved/resized for PID: {}", context.pid);
        Some(WindowEvent::LocationChange(window))
      }
      kAXWindowMiniaturizedNotification => {
        tracing::info!("Window minimized for PID: {}", context.pid);
        Some(WindowEvent::Minimize(window))
      }
      kAXWindowDeminiaturizedNotification => {
        tracing::info!("Window deminimized for PID: {}", context.pid);
        Some(WindowEvent::MinimizeEnd(window))
      }
      kAXTitleChangedNotification => {
        tracing::debug!("Window title changed for PID: {}", context.pid);
        Some(WindowEvent::TitleChange(window))
      }
      kAXMainWindowChangedNotification => {
        tracing::debug!("Main window changed for PID: {}", context.pid);
        Some(WindowEvent::Focus(window))
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

impl Drop for ApplicationObserver {
  fn drop(&mut self) {
    tracing::debug!("Cleaning up AppWindowObserver for PID {}", self.pid);

    // Remove all notifications.
    // for notification in AX_APP_NOTIFICATIONS {
    //   unsafe {
    //     let notification_cfstr =
    // CFString::from_static_str(notification);     self
    //       .observer
    //       .remove_notification(&self.app_element, &notification_cfstr);
    //   }
    // }

    // Remove from run loop.
    unsafe {
      if let Some(runloop) = CFRunLoop::current() {
        runloop.remove_source(
          Some(&self.observer_source),
          kCFRunLoopDefaultMode,
        );
      }
    }

    tracing::debug!(
      "AppWindowObserver cleanup completed for PID {}",
      self.pid
    );
  }
}
