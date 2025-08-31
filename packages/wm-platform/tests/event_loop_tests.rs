use std::panic::{self, AssertUnwindSafe};

use wm_platform::EventLoop;

/// Integration tests for EventLoop functionality.
/// These tests are separate from unit tests because they require main
/// thread execution on macOS, which conflicts with the standard test
/// harness.

struct TestResult {
  name: &'static str,
  passed: bool,
  error: Option<String>,
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

fn test_event_loop_creation() {
  let result = EventLoop::new();
  if let Err(err) = result {
    panic!("EventLoop creation failed: {:?}", err);
  }
}

fn test_event_loop_new_returns_both_loop_and_dispatcher() {
  let result = EventLoop::new();
  assert!(result.is_ok(), "EventLoop::new should succeed");

  let (_event_loop, _dispatcher) = result.unwrap();
  // Both should be created successfully - no need to test internal state
}

fn main() {
  println!("Running event loop integration tests...");
  
  let tests: Vec<(&str, fn())> = vec![
    ("test_event_loop_creation", test_event_loop_creation),
    ("test_event_loop_new_returns_both_loop_and_dispatcher", test_event_loop_new_returns_both_loop_and_dispatcher),
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