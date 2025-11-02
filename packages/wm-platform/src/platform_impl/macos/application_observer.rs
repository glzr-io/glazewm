use std::{
  ptr::NonNull,
  sync::{Arc, Mutex},
};

use objc2_application_services::{AXError, AXObserver, AXUIElement};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource, CFString,
};
use tokio::sync::mpsc;

use crate::{
  platform_impl::{Application, NativeWindow, ProcessId},
  NativeWindowExtMacOs, ThreadBound, WindowEvent, WindowId,
};

/// Notifications to register for the `AXUIElement` of an application.
const AX_APP_NOTIFICATIONS: &[&str] =
  &["AXFocusedWindowChanged", "AXWindowCreated"];

/// Notifications to register for the `AXUIElement` of a window.
const AX_WINDOW_NOTIFICATIONS: &[&str] = &[
  "AXTitleChanged",
  "AXUIElementDestroyed",
  "AXWindowMoved",
  "AXWindowResized",
  "AXWindowDeminiaturized",
  "AXWindowMiniaturized",
];

/// Context passed to the application event callback.
#[derive(Debug)]
struct ApplicationEventContext {
  application: Application,
  events_tx: mpsc::UnboundedSender<WindowEvent>,
  app_windows: Arc<Mutex<Vec<crate::NativeWindow>>>,
  observer: CFRetained<AXObserver>,
}

/// Represents an accessibility observer for a specific application.
#[derive(Debug)]
pub(crate) struct ApplicationObserver {
  pub(crate) pid: ProcessId,
  app_windows: Arc<Mutex<Vec<crate::NativeWindow>>>,
  events_tx: mpsc::UnboundedSender<WindowEvent>,
  observer: CFRetained<AXObserver>,
  observer_source: CFRetained<CFRunLoopSource>,
  app_element: Arc<ThreadBound<CFRetained<AXUIElement>>>,
  // context: Box<ApplicationEventContext>,
}

// TODO: Remove this.
unsafe impl Send for ApplicationObserver {}

impl ApplicationObserver {
  /// Creates a new `ApplicationObserver` for the given application.
  ///
  /// If `is_startup` is `true`, the observer will not emit
  /// `WindowEvent::Show` for windows already running on startup.
  pub fn new(
    app: &Application,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    is_startup: bool,
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

    let app_windows = Arc::new(Mutex::new(app.windows()?));
    let context = Box::into_raw(Box::new(ApplicationEventContext {
      application: app.clone(),
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
    for window in app_windows.lock().unwrap().iter() {
      if let Err(err) =
        Self::register_window_notifications(window, &observer, context)
      {
        tracing::warn!(
          "Failed to register window notifications for PID {}: {}",
          app.pid,
          err
        );
      }

      // Don't emit `WindowEvent::Show` for windows that are already
      // running on startup.
      if !is_startup {
        if let Err(err) = events_tx.send(WindowEvent::Show {
          window: window.clone(),
          notification: crate::WindowEventNotification(None),
        }) {
          tracing::warn!(
            "Failed to send window event for PID {}: {}",
            app.pid,
            err
          );
        }
      }
    }

    Ok(Self {
      pid: app.pid,
      app_windows,
      events_tx,
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
    for notification in AX_APP_NOTIFICATIONS {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        let result = observer.add_notification(
          app.ax_element.get_ref()?,
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
    for notification in AX_WINDOW_NOTIFICATIONS {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        let result = observer.add_notification(
          window.ax_ui_element().get_ref()?,
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

  pub(crate) fn emit_all_windows_destroyed(&self) {
    for window in self.app_windows.lock().unwrap().iter() {
      if let Err(err) = self.events_tx.send(WindowEvent::Destroy {
        window_id: window.id(),
        notification: crate::WindowEventNotification(None),
      }) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          self.pid,
          err
        );
      }
    }
  }

  pub(crate) fn emit_all_windows_hidden(&self) {
    for window in self.app_windows.lock().unwrap().iter() {
      if let Err(err) = self.events_tx.send(WindowEvent::Hide {
        window: window.clone(),
        notification: crate::WindowEventNotification(None),
      }) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          self.pid,
          err
        );
      }
    }
  }

  pub(crate) fn emit_all_windows_shown(&self) {
    for window in self.app_windows.lock().unwrap().iter() {
      if let Err(err) = self.events_tx.send(WindowEvent::Show {
        window: window.clone(),
        notification: crate::WindowEventNotification(None),
      }) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          self.pid,
          err
        );
      }
    }
  }

  /// Callback function for accessibility window events.
  #[allow(clippy::too_many_lines)]
  unsafe extern "C-unwind" fn window_event_callback(
    _observer: NonNull<AXObserver>,
    element: NonNull<AXUIElement>,
    notification_name: NonNull<CFString>,
    context: *mut std::ffi::c_void,
  ) {
    if context.is_null() {
      tracing::error!("Window event callback received null context.");
      return;
    }

    let context = &mut *context.cast::<ApplicationEventContext>();
    let ax_element = unsafe { CFRetained::retain(element) };
    let notification = crate::WindowEventNotificationInner {
      name: notification_name.as_ref().to_string(),
      ax_element_ptr: element.as_ptr().cast::<std::ffi::c_void>(),
    };

    tracing::debug!(
      "Received window event: {} for PID: {}",
      notification.name,
      context.application.pid
    );

    let found_window = {
      let app_windows = context.app_windows.lock().unwrap();

      app_windows
        .iter()
        .find(|window| {
          window.ax_ui_element().get_ref().ok() == Some(&ax_element)
        })
        .cloned()
    };

    if notification.name.as_str() == "AXUIElementDestroyed" {
      if let Some(window) = &found_window {
        context
          .app_windows
          .lock()
          .unwrap()
          .retain(|w| w.id() != window.id());

        if let Err(err) = context.events_tx.send(WindowEvent::Destroy {
          window_id: window.id(),
          notification: crate::WindowEventNotification(Some(notification)),
        }) {
          tracing::warn!(
            "Failed to send window event for PID {}: {}",
            context.application.pid,
            err
          );
        }
      }

      return;
    }

    let is_new_window = found_window.is_none();
    let window = found_window.unwrap_or_else(|| {
      let window_id = WindowId::from_window_element(&ax_element);
      let ax_element = ThreadBound::new(
        ax_element,
        context.application.dispatcher.clone(),
      );
      NativeWindow::new(window_id, ax_element, context.application.clone())
        .into()
    });

    if is_new_window {
      context.app_windows.lock().unwrap().push(window.clone());
      let _ = Self::register_window_notifications(
        &window,
        &context.observer.clone(),
        context,
      );

      if let Err(err) = context.events_tx.send(WindowEvent::Show {
        window: window.clone(),
        notification: crate::WindowEventNotification(Some(
          notification.clone(),
        )),
      }) {
        tracing::warn!(
          "Failed to send window event for PID {}: {}",
          context.application.pid,
          err
        );
      }
    }

    let window_event = match notification.name.as_str() {
      "AXFocusedWindowChanged" => WindowEvent::Focus {
        window,
        notification: crate::WindowEventNotification(Some(notification)),
      },
      "AXWindowMoved" | "AXWindowResized" => WindowEvent::MoveOrResize {
        window,
        is_interactive_start: false,
        is_interactive_end: false,
        notification: crate::WindowEventNotification(Some(notification)),
      },
      "AXWindowMiniaturized" => WindowEvent::Minimize {
        window,
        notification: crate::WindowEventNotification(Some(notification)),
      },
      "AXWindowDeminiaturized" => WindowEvent::MinimizeEnd {
        window,
        notification: crate::WindowEventNotification(Some(notification)),
      },
      "AXTitleChanged" => WindowEvent::TitleChange {
        window,
        notification: crate::WindowEventNotification(Some(notification)),
      },
      _ => {
        tracing::debug!(
          "Unhandled window notification: {} for PID: {}",
          notification.name,
          context.application.pid
        );
        return;
      }
    };

    if let Err(err) = context.events_tx.send(window_event) {
      tracing::warn!(
        "Failed to send window event for PID {}: {}",
        context.application.pid,
        err
      );
    }
  }
}

impl Drop for ApplicationObserver {
  fn drop(&mut self) {
    tracing::debug!("Cleaning up AppWindowObserver for PID {}", self.pid);

    // TODO: **** The following needs to be run on the thread in which the
    // observer was created.

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
