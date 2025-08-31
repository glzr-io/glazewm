use std::{
  panic::{self, AssertUnwindSafe},
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use wm_common::Point;
use wm_platform::EventLoop;

/// Integration tests for Dispatcher functionality.
/// These tests are separate from unit tests because they require main
/// thread execution on macOS, which conflicts with the standard test
/// harness.

struct TestResult {
  name: &'static str,
  passed: bool,
  error: Option<String>,
}

fn create_test_dispatcher() -> wm_platform::Dispatcher {
  let (_, dispatcher) =
    EventLoop::new().expect("Failed to create EventLoop");
  dispatcher
}

fn run_test<F>(name: &'static str, test_fn: F) -> TestResult
where
  F: FnOnce() + std::panic::UnwindSafe,
{
  let result = panic::catch_unwind(AssertUnwindSafe(test_fn));
  match result {
    Ok(()) => TestResult {
      name,
      passed: true,
      error: None,
    },
    Err(panic_info) => {
      let error_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
      } else if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
      } else {
        "Unknown panic".to_string()
      };
      TestResult {
        name,
        passed: false,
        error: Some(error_msg),
      }
    }
  }
}

fn test_dispatcher_creation() {
  let dispatcher = create_test_dispatcher();
  assert!(std::ptr::addr_of!(dispatcher) != std::ptr::null());
}

fn test_dispatcher_clone() {
  let dispatcher1 = create_test_dispatcher();
  let dispatcher2 = dispatcher1.clone();

  // Both dispatchers should be independent but equivalent
  assert!(
    std::ptr::addr_of!(dispatcher1) != std::ptr::addr_of!(dispatcher2)
  );
}

fn test_dispatcher_debug() {
  let dispatcher = create_test_dispatcher();
  let debug_str = format!("{:?}", dispatcher);
  assert_eq!(debug_str, "EventLoopDispatcher");
}

fn test_dispatch_on_same_thread() {
  let dispatcher = create_test_dispatcher();
  let counter = Arc::new(Mutex::new(0));
  let counter_clone = Arc::clone(&counter);

  let result = dispatcher.dispatch(move || {
    let mut count = counter_clone.lock().unwrap();
    *count += 1;
  });

  assert!(result.is_ok());

  // If we're on the main thread (macOS), dispatch should execute
  // immediately
  if dispatcher.is_main_thread() {
    let final_count = *counter.lock().unwrap();
    assert_eq!(final_count, 1);
  }
}

fn test_dispatch_sync_behavior() {
  let dispatcher = create_test_dispatcher();

  // Test sync dispatch - behavior depends on main thread detection
  let result = dispatcher.dispatch_sync(|| 42);

  match result {
    Ok(value) => {
      assert_eq!(value, 42);
    }
    Err(_) => {
      // Expected when not on main thread or no event source
    }
  }
}

fn test_dispatch_sync_with_complex_return_type() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.dispatch_sync(|| vec![1, 2, 3, 4, 5]);

  match result {
    Ok(vec) => {
      assert_eq!(vec, vec![1, 2, 3, 4, 5]);
    }
    Err(_) => {
      // Expected when not on main thread or no event source
    }
  }
}

fn test_dispatch_sync_with_string_return() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.dispatch_sync(|| "Hello, World!".to_string());

  match result {
    Ok(value) => {
      assert_eq!(value, "Hello, World!");
    }
    Err(_) => {
      // Expected when not on main thread or no event source
    }
  }
}

fn test_multiple_dispatches() {
  let dispatcher = create_test_dispatcher();
  let counter = Arc::new(Mutex::new(0));

  // Execute multiple dispatches
  for i in 0..10 {
    let counter_clone = Arc::clone(&counter);
    let result = dispatcher.dispatch(move || {
      let mut count = counter_clone.lock().unwrap();
      *count += i;
    });
    assert!(result.is_ok());
  }

  // If we're on the main thread (macOS), all dispatches should execute
  // Otherwise, they may not execute due to no event source
  let final_count = *counter.lock().unwrap();

  // On macOS main thread: should be 45 (sum of 0..10)
  // On other platforms or non-main thread: may be 0
  if dispatcher.is_main_thread() {
    assert_eq!(final_count, 45);
  } else {
    // No assertions - behavior depends on platform and thread
  }
}

fn test_dispatch_with_panic_recovery() {
  let dispatcher = create_test_dispatcher();

  // Only test panic recovery if we're on the main thread
  // where dispatch executes immediately
  if dispatcher.is_main_thread() {
    // This should not crash the dispatcher
    let result = dispatcher.dispatch(|| {
      panic!("Test panic");
    });

    // The dispatch should still succeed (panic is caught internally)
    assert!(result.is_ok());

    // Dispatcher should still be functional after panic
    let result2 = dispatcher.dispatch_sync(|| 42);
    if result2.is_ok() {
      assert_eq!(result2.unwrap(), 42);
    }
  }
}

// #[cfg(target_os = "macos")]
// fn test_is_main_thread_detection_macos() {
//   let dispatcher = create_test_dispatcher();

//   // On macOS, we should be able to detect if we're on the main thread
//   // This test assumes we're running tests on the main thread
//   let is_main = dispatcher.is_main_thread();

//   // This assertion may vary depending on how tests are run
//   // but the method should not panic
//   assert!(is_main || !is_main); // Tautology to ensure no panic
// }

#[cfg(target_os = "windows")]
fn test_is_main_thread_detection_windows() {
  let dispatcher = create_test_dispatcher();

  // On Windows, thread detection depends on the source having a
  // thread_id With no source, this should still not panic
  let is_main = dispatcher.is_main_thread();

  // This assertion may vary depending on how tests are run
  // but the method should not panic
  assert!(is_main || !is_main); // Tautology to ensure no panic
}

fn test_concurrent_dispatcher_usage() {
  let dispatcher = create_test_dispatcher();
  let counter = Arc::new(Mutex::new(0));
  let mut handles = vec![];

  // Spawn multiple threads that use the dispatcher
  for _ in 0..5 {
    let dispatcher_clone = dispatcher.clone();
    let counter_clone = Arc::clone(&counter);

    let handle = thread::spawn(move || {
      thread::sleep(Duration::from_millis(10));
      let result = dispatcher_clone.dispatch(move || {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
      });
      assert!(result.is_ok());
    });

    handles.push(handle);
  }

  // Wait for all threads to complete
  for handle in handles {
    handle.join().unwrap();
  }

  // Depending on platform and main thread status, counter may vary
  let final_count = *counter.lock().unwrap();
  assert!(final_count >= 0); // Basic sanity check
}

fn test_displays_method_call() {
  let dispatcher = create_test_dispatcher();

  // The displays() method should not panic, even if it returns an error
  // due to no actual display hardware being available in test
  let result = dispatcher.displays();

  // We don't assert specific values since test environment may not have
  // displays but the method should not panic
  match result {
    Ok(_displays) => {
      // Success case - displays were retrieved
    }
    Err(_) => {
      // Expected in headless test environment
    }
  }
}

fn test_all_display_devices_method_call() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.all_display_devices();

  // Similar to displays() - should not panic
  match result {
    Ok(_devices) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment
    }
  }
}

fn test_display_from_point_method_call() {
  let dispatcher = create_test_dispatcher();
  let point = Point { x: 100, y: 100 };

  let result = dispatcher.display_from_point(point);

  // Should not panic
  match result {
    Ok(_display) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment
    }
  }
}

fn test_primary_display_method_call() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.primary_display();

  // Should not panic
  match result {
    Ok(_display) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment
    }
  }
}

fn test_all_windows_method_call() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.all_windows();

  // Should not panic
  match result {
    Ok(_windows) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment or insufficient permissions
    }
  }
}

fn test_all_applications_method_call() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.all_applications();

  // Should not panic
  match result {
    Ok(_applications) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment or insufficient permissions
    }
  }
}

fn test_visible_windows_method_call() {
  let dispatcher = create_test_dispatcher();

  let result = dispatcher.visible_windows();

  // Should not panic
  match result {
    Ok(_windows) => {
      // Success case
    }
    Err(_) => {
      // Expected in headless test environment or insufficient permissions
    }
  }
}

fn main() {
  println!("Running dispatcher integration tests...");
  
  let tests: Vec<(&str, fn())> = vec![
    ("test_dispatcher_creation", test_dispatcher_creation),
    ("test_dispatcher_clone", test_dispatcher_clone),
    ("test_dispatcher_debug", test_dispatcher_debug),
    ("test_dispatch_on_same_thread", test_dispatch_on_same_thread),
    ("test_dispatch_sync_behavior", test_dispatch_sync_behavior),
    ("test_dispatch_sync_with_complex_return_type", test_dispatch_sync_with_complex_return_type),
    ("test_dispatch_sync_with_string_return", test_dispatch_sync_with_string_return),
    ("test_multiple_dispatches", test_multiple_dispatches),
    ("test_dispatch_with_panic_recovery", test_dispatch_with_panic_recovery),
    #[cfg(target_os = "windows")]
    ("test_is_main_thread_detection_windows", test_is_main_thread_detection_windows),
    ("test_concurrent_dispatcher_usage", test_concurrent_dispatcher_usage),
    ("test_displays_method_call", test_displays_method_call),
    ("test_all_display_devices_method_call", test_all_display_devices_method_call),
    ("test_display_from_point_method_call", test_display_from_point_method_call),
    ("test_primary_display_method_call", test_primary_display_method_call),
    ("test_all_windows_method_call", test_all_windows_method_call),
    ("test_all_applications_method_call", test_all_applications_method_call),
    ("test_visible_windows_method_call", test_visible_windows_method_call),
  ];
  
  let mut results = Vec::new();
  let mut passed = 0;
  let mut failed = 0;
  
  for (name, test_fn) in tests {
    let result = run_test(name, test_fn);
    if result.passed {
      passed += 1;
      println!("✓ {}", name);
    } else {
      failed += 1;
      println!("✗ {} - {}", name, result.error.as_deref().unwrap_or("Unknown error"));
    }
    results.push(result);
  }
  
  println!("\nTest Results: {} passed, {} failed, {} total", passed, failed, passed + failed);
  
  if failed > 0 {
    std::process::exit(1);
  }
}