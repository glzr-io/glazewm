use std::{os::raw::c_void, ptr::NonNull};

use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};

use crate::{Dispatcher, Error, Key, KeyCode, ThreadBound};

/// A key event received from the keyboard hook.
#[derive(Clone, Debug)]
pub struct KeyEvent {
  /// The key that was pressed or released.
  pub key: Key,

  /// Key code that generated this event.
  #[allow(dead_code)]
  pub key_code: KeyCode,

  /// Whether the event is for a key press or release.
  pub is_keypress: bool,

  /// Modifier key flags at the time of the event.
  event_flags: CGEventFlags,
}

impl KeyEvent {
  /// Gets whether the specified key is currently pressed.
  pub fn is_key_down(&self, key: Key) -> bool {
    match key {
      Key::Cmd
      | Key::LCmd
      | Key::RCmd
      | Key::Win
      | Key::LWin
      | Key::RWin => {
        self.event_flags & CGEventFlags::MaskCommand
          != CGEventFlags::empty()
      }
      Key::Alt | Key::LAlt | Key::RAlt => {
        self.event_flags & CGEventFlags::MaskAlternate
          != CGEventFlags::empty()
      }
      Key::Ctrl | Key::LCtrl | Key::RCtrl => {
        self.event_flags & CGEventFlags::MaskControl
          != CGEventFlags::empty()
      }
      Key::Shift | Key::LShift | Key::RShift => {
        self.event_flags & CGEventFlags::MaskShift != CGEventFlags::empty()
      }
      _ => {
        // TODO: For non-modifier keys, check using CGEventSourceStateID.
        false
      }
    }
  }
}

/// Data shared with the `CGEventTap` callback.
struct CallbackData {
  callback: Box<dyn Fn(KeyEvent) -> bool + Send + Sync + 'static>,
}

/// A system-wide low-level keyboard hook.
#[derive(Debug)]
pub struct KeyboardHook {
  /// Mach port for the created `CGEventTap`.
  tap_port: Option<ThreadBound<CFRetained<CFMachPort>>>,

  /// Pointer to [`CallbackData`], used by the `CGEventTap` callback.
  callback_ptr: Option<usize>,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  ///
  /// The callback is called for every keyboard event and returns `true` if
  /// the event should be intercepted.
  pub fn new<F>(
    callback: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self>
  where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
  {
    let callback_ptr = {
      let data = Box::new(CallbackData {
        callback: Box::new(callback),
      });
      Box::into_raw(data) as usize
    };

    let tap_port = dispatcher
      .dispatch_sync(|| Self::create_event_tap(callback_ptr, dispatcher))
      .flatten()
      .inspect_err(|_| {
        // Clean up the callback data if event tap creation fails.
        let _ =
          unsafe { Box::from_raw(callback_ptr as *mut CallbackData) };
      })?;

    Ok(Self {
      tap_port: Some(tap_port),
      callback_ptr: Some(callback_ptr),
    })
  }

  /// Terminates the keyboard hook by invalidating the event tap.
  #[allow(clippy::unnecessary_wraps)]
  pub fn terminate(&mut self) -> crate::Result<()> {
    if let Some(tap) = self.tap_port.take() {
      // Invalidate the event tap to stop it from receiving events. This
      // also invalidates the run loop source.
      // See: https://developer.apple.com/documentation/corefoundation/cfmachportinvalidate(_:)
      let _ = tap.with(|tap| CFMachPort::invalidate(tap));
    }

    // Clean up the callback data if it exists.
    if let Some(ptr) = self.callback_ptr.take() {
      let _ = unsafe { Box::from_raw(ptr as *mut CallbackData) };
    }

    Ok(())
  }

  /// Creates a `CGEventTap` object.
  fn create_event_tap(
    callback_ptr: usize,
    dispatcher: &Dispatcher,
  ) -> crate::Result<ThreadBound<CFRetained<CFMachPort>>> {
    let mask: CGEventMask = (1u64 << u64::from(CGEventType::KeyDown.0))
      | (1u64 << u64::from(CGEventType::KeyUp.0));

    let tap_port = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::SessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        mask,
        Some(Self::keyboard_event_callback),
        callback_ptr as *mut c_void,
      )
      .ok_or_else(|| {
        Error::Platform(
          "Failed to create `CGEventTap`. Accessibility permissions may be required."
            .to_string(),
        )
      })
    }?;

    let loop_source =
      CFMachPort::new_run_loop_source(None, Some(&tap_port), 0)
        .ok_or_else(|| {
          Error::Platform("Failed to create loop source".to_string())
        })?;

    let current_loop = CFRunLoop::current().ok_or_else(|| {
      Error::Platform("Failed to get current run loop".to_string())
    })?;

    current_loop
      .add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

    CGEvent::tap_enable(&tap_port, true);

    Ok(ThreadBound::new(tap_port, dispatcher.clone()))
  }

  /// Callback function for keyboard events.
  ///
  /// For use with `CGEventTap`.
  extern "C-unwind" fn keyboard_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
  ) -> *mut CGEvent {
    if user_info.is_null() {
      tracing::error!("Null pointer passed to keyboard event callback.");
      return unsafe { event.as_mut() };
    }

    // Extract the key code of the pressed/released key.
    let key_code = KeyCode(unsafe {
      CGEvent::integer_value_field(
        Some(event.as_ref()),
        CGEventField::KeyboardEventKeycode,
      )
    });

    // Try to convert the key code to a known key.
    let Ok(key) = Key::try_from(key_code) else {
      return unsafe { event.as_mut() };
    };

    let event_flags = unsafe { CGEvent::flags(Some(event.as_ref())) };
    let key_event = KeyEvent {
      key,
      key_code,
      is_keypress: event_type == CGEventType::KeyDown,
      event_flags,
    };

    // Get callback from user data and invoke it.
    let data = unsafe { &*(user_info as *const CallbackData) };
    let should_intercept = (data.callback)(key_event);

    if should_intercept {
      std::ptr::null_mut()
    } else {
      unsafe { event.as_mut() }
    }
  }
}

impl Drop for KeyboardHook {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
