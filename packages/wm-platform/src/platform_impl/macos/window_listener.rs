use accessibility_sys::{
  kAXMainWindowChangedNotification, kAXTitleChangedNotification,
  kAXUIElementDestroyedNotification, kAXWindowCreatedNotification,
  kAXWindowDeminiaturizedNotification, kAXWindowMiniaturizedNotification,
  kAXWindowMovedNotification, kAXWindowResizedNotification,
};
use anyhow::Context;
use objc2_app_kit::{NSApplication, NSWorkspace};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFBoolean, CFRetained, CFRunLoop, CFString,
};
use objc2_foundation::MainThreadMarker;
use tokio::sync::mpsc;

use crate::{
  platform_impl::{
    classes::{
      NotificationCenter, NotificationEvent, NotificationName,
      NotificationObserver,
    },
    AXObserverAddNotification, AXObserverCreate,
    AXObserverGetRunLoopSource, AXObserverRef, AXUIElement,
    AXUIElementCreateApplication, AXUIElementExt, AXUIElementRef,
    CFStringRef, EventLoopDispatcher, MainThreadRef, NativeWindow,
    ProcessId,
  },
  WindowEvent,
};

/// Represents an accessibility observer for a specific application
#[derive(Debug)]
struct AppWindowObserver {
  // observer: AXObserverRef,
  // pid: pid_t,
  // app_element: AXUIElement,
  // _runloop_source: objc2_core_foundation::CFRetained<CFRunLoopSource>,
}

impl Drop for AppWindowObserver {
  fn drop(&mut self) {
    // Remove notifications and cleanup
    let window_notifications = [
      kAXWindowCreatedNotification,
      kAXUIElementDestroyedNotification,
      kAXWindowMovedNotification,
      kAXWindowResizedNotification,
      kAXWindowMiniaturizedNotification,
      kAXWindowDeminiaturizedNotification,
      kAXTitleChangedNotification,
      kAXMainWindowChangedNotification,
    ];

    for notification in &window_notifications {
      let _notification_cfstr = CFString::from_str(notification);
      // AXObserverRemoveNotification(
      //   self.observer,
      //   (&self.app_element as *const AXUIElement) as *mut _,
      //   (&*notification_cfstr as *const CFString) as *const _,
      // );
    }
  }
}

#[derive(Debug)]
pub struct WindowListener {
  pub event_rx: mpsc::UnboundedReceiver<WindowEvent>,
}

impl WindowListener {
  pub fn new(dispatcher: &EventLoopDispatcher) -> anyhow::Result<Self> {
    let (events_tx, event_rx) = mpsc::unbounded_channel();

    let dispatcher_clone = dispatcher.clone();
    dispatcher.dispatch_sync(|| {
      Self::add_observers(events_tx, dispatcher_clone);
    })?;

    Ok(Self { event_rx })
  }

  fn add_observers(
    events_tx: mpsc::UnboundedSender<WindowEvent>,
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

    std::thread::spawn(move || {
      Self::listen(events_rx, events_tx, dispatcher);
    });

    // TODO: Hack to prevent the handler from being deregistered.
    std::mem::forget(observer);
    std::mem::forget(workspace_center);
    std::mem::forget(default_center);
  }

  fn listen(
    mut events_rx: mpsc::UnboundedReceiver<NotificationEvent>,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: EventLoopDispatcher,
  ) {
    // Track window observers for each application by PID
    // let mut app_observers: HashMap<pid_t, AppWindowObserver> =
    //   HashMap::new();

    while let Some(event) = events_rx.blocking_recv() {
      tracing::info!("Received event: {event:?}");

      match event {
        NotificationEvent::WorkspaceDidLaunchApplication(running_app) => {
          tracing::info!("Workspace launched application.");

          let dispatcher_clone = dispatcher.clone();
          let events_tx_clone = events_tx.clone();
          let _ = dispatcher.dispatch_sync(move || {
            // Register window event listeners for the new application
            let pid = unsafe { running_app.processIdentifier() };

            match Self::register_window_observer(
              pid,
              events_tx_clone.clone(),
              &dispatcher_clone,
            ) {
              Ok(_observer) => {
                tracing::info!(
                  "Successfully registered window observer for PID: {}",
                  pid
                );
                // app_observers.insert(pid, observer);
              }
              Err(err) => {
                tracing::warn!(
                  "Failed to register window observer for PID {}: {}",
                  pid,
                  err
                );
              }
            }
          });
        }
        NotificationEvent::WorkspaceDidTerminateApplication(
          running_app,
        ) => {
          tracing::info!("Workspace terminated application.");

          // Clean up window observers for the terminated application
          let _pid = unsafe { running_app.processIdentifier() };

          // if let Some(observer) = app_observers.remove(&pid) {
          //   tracing::info!(
          //     "Removed window observer for terminated PID: {}",
          //     pid
          //   );
          //   drop(observer); // This will trigger the cleanup in Drop
          //                   // implementation
          // }
        }
        _ => {}
      }
    }
  }

  /// Register an accessibility observer for window events on the specified
  /// application
  fn register_window_observer(
    pid: ProcessId,
    events_tx: mpsc::UnboundedSender<WindowEvent>,
    dispatcher: &EventLoopDispatcher,
  ) -> anyhow::Result<AppWindowObserver> {
    // NOTE: Accessibility APIs must be called directly on main thread
    // dispatch_sync can cause threading issues with the accessibility
    // system

    // Create the application AX element using low-level API
    let app_element = unsafe { AXUIElementCreateApplication(pid) };

    // Create an accessibility observer for this application
    let mut observer: AXObserverRef = std::ptr::null_mut();
    let result = unsafe {
      AXObserverCreate(pid, window_event_callback, &mut observer)
    };

    if result != 0 {
      return Err(anyhow::anyhow!(
        "Failed to create AX observer for PID {}: {}",
        pid,
        result
      ));
    }

    println!("got here1");

    // Set up the callback context (we'll store the events_tx here)
    let context = Box::into_raw(Box::new(WindowEventContext {
      events_tx,
      dispatcher: dispatcher.clone(),
      pid,
    }));

    // Store the context pointer in the observer (this is a simplified
    // approach) In a real implementation, you'd want to use a global
    // registry to map observers to contexts

    println!("got here2");
    // Get the run loop source and add it to the current run loop
    let runloop_source = unsafe {
      let source = AXObserverGetRunLoopSource(observer);
      CFRetained::retain(std::ptr::NonNull::new_unchecked(
        source as *mut _,
      ))
    };

    println!("got here2.5");

    unsafe {
      let runloop =
        CFRunLoop::current().context("Failed to get current runloop")?;
      runloop.add_source(Some(&runloop_source), kCFRunLoopDefaultMode);
    }
    println!("got here2.6");

    // Register for window notifications
    let window_notifications = [
      kAXWindowCreatedNotification,
      kAXUIElementDestroyedNotification,
      kAXWindowMovedNotification,
      kAXWindowResizedNotification,
      kAXWindowMiniaturizedNotification,
      kAXWindowDeminiaturizedNotification,
      kAXTitleChangedNotification,
      kAXMainWindowChangedNotification,
    ];

    for notification in &window_notifications {
      unsafe {
        let notification_cfstr = CFString::from_static_str(notification);
        println!("got here2.7");

        let result = AXObserverAddNotification(
          observer,
          app_element,
          &notification_cfstr,
          context as *mut std::ffi::c_void,
        );

        println!("got here2.8");

        if result != 0 {
          tracing::warn!(
            "Failed to add notification {} for PID {}: {}",
            notification,
            pid,
            result
          );
        }
      }
    }

    println!("got here3");
    Ok(AppWindowObserver {
      // observer,
      // pid,
      // app_element,
      // _runloop_source: runloop_source,
    })
  }

  /// Returns the next event from the `WindowListener`.
  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    self.event_rx.recv().await
  }
}

/// Context data passed to the window event callback
struct WindowEventContext {
  events_tx: mpsc::UnboundedSender<WindowEvent>,
  dispatcher: EventLoopDispatcher,
  pid: ProcessId,
}

/// Callback function for accessibility window events
#[allow(clippy::too_many_lines)]
unsafe extern "C" fn window_event_callback(
  _observer: AXObserverRef,
  element: AXUIElementRef,
  notification: CFStringRef,
  context: *mut std::ffi::c_void,
) {
  println!("got here4");
  if context.is_null() {
    println!("got here4.1");
    tracing::error!("Window event callback received null context");
    return;
  }

  println!("got here4.2");
  let context = &*(context as *const WindowEventContext);
  let cf_string: CFRetained<CFString> = unsafe {
    CFRetained::retain(std::ptr::NonNull::new_unchecked(
      notification as *mut _,
    ))
  };
  let notification_str = cf_string.to_string();
  println!("got here4.3");

  // Retain the element for safe access
  let ax_element = match unsafe { AXUIElement::from_ref(element) } {
    Ok(el) => el,
    Err(err) => {
      tracing::error!("Failed to retain AXUIElement in callback: {}", err);
      return;
    }
  };

  // Get window title using generic attribute API
  match ax_element.get_attribute::<CFString>("AXTitle") {
    Ok(cf_title) => {
      let title = cf_title.to_string();
      println!("Window title: '{}'", title);
      tracing::debug!(
        "Window title: '{}' for PID: {}",
        title,
        context.pid
      );
    }
    Err(err) => {
      println!("Failed to get window title: {}", err);
      tracing::debug!(
        "Failed to get window title for PID {}: {}",
        context.pid,
        err
      );
    }
  }

  // Example: Get additional window attributes
  if let Ok(cf_minimized) =
    ax_element.get_attribute::<CFBoolean>("AXMinimized")
  {
    let is_minimized = cf_minimized.value();
    tracing::debug!(
      "Window minimized state: {} for PID: {}",
      is_minimized,
      context.pid
    );
  }

  // TODO: Implement AXValue extraction for frame/position

  tracing::info!(
    "Received window event: {} for PID: {}",
    notification_str,
    context.pid
  );

  let ax_element_ref =
    MainThreadRef::new(context.dispatcher.clone(), ax_element);

  let window =
    NativeWindow::new(0, context.dispatcher.clone(), ax_element_ref);

  // println!("Window title 222: '{:?}'", window.title());

  let window_event = match notification_str.as_str() {
    kAXWindowCreatedNotification => {
      tracing::info!("Window created for PID: {}", context.pid);
      Some(WindowEvent::Show(window.into()))
    }
    kAXUIElementDestroyedNotification => {
      tracing::info!("Window destroyed for PID: {}", context.pid);
      Some(WindowEvent::Hide(window.into()))
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

// TODO: Implement get_attribute function when needed
// This function was corrupted and is commented out for now
