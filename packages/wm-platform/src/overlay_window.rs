use objc2::{AnyThread, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSFloatingWindowLevel, NSImage,
  NSImageView, NSScreen, NSWindow, NSWindowStyleMask,
};
use objc2_core_foundation::{CGRect, CGSize};
#[allow(deprecated)]
use objc2_core_graphics::{
  CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption,
};
use objc2_foundation::NSRect;

use crate::{Dispatcher, Rect, ThreadBound, WindowId};

/// A borderless overlay `NSWindow` displaying a screenshot of a real
/// window.
///
/// Used for smooth animations — moving our own window is much cheaper than
/// AX API calls on 3rd-party windows.
pub struct OverlayWindow {
  ns_window: ThreadBound<objc2::rc::Retained<NSWindow>>,
}

impl OverlayWindow {
  /// Screenshots the window and creates an overlay `NSWindow` at
  /// `initial_rect`.
  #[allow(deprecated)] // CGWindowListCreateImage is deprecated but functional.
  pub fn new(
    window_id: WindowId,
    initial_rect: &Rect,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let wid = window_id.0;
    let rect = initial_rect.clone();
    let disp = dispatcher.clone();

    let ns_window = dispatcher.dispatch_sync(move || {
      // SAFETY: `dispatch_sync` executes on the event loop (main) thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      // Screenshot the target window.
      let cg_rect = CGRect::new(
        objc2_core_foundation::CGPoint {
          x: f64::from(rect.x()),
          y: f64::from(rect.y()),
        },
        CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      let cg_image = CGWindowListCreateImage(
        cg_rect,
        CGWindowListOption::OptionIncludingWindow,
        wid,
        CGWindowImageOption::BoundsIgnoreFraming
          | CGWindowImageOption::BestResolution,
      );

      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint {
          x: f64::from(rect.x()),
          y: flipped_y(&rect, mtm),
        },
        objc2_foundation::NSSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      // Create borderless NSWindow.
      let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          NSWindow::alloc(mtm),
          ns_rect,
          NSWindowStyleMask::Borderless,
          NSBackingStoreType::Buffered,
          false,
        )
      };

      window.setBackgroundColor(Some(&NSColor::clearColor()));
      window.setOpaque(false);
      window.setHasShadow(false);
      window.setLevel(NSFloatingWindowLevel);
      // SAFETY: We own this window and manage its lifetime via
      // `ThreadBound` + `orderOut`.
      unsafe { window.setReleasedWhenClosed(false) };

      // Build the image content from the screenshot.
      if let Some(cg_image) = cg_image {
        let logical_size = CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        };
        let ns_image = NSImage::initWithCGImage_size(
          NSImage::alloc(),
          &cg_image,
          logical_size,
        );

        let image_view = NSImageView::imageViewWithImage(&ns_image, mtm);
        window.setContentView(Some(&image_view));
      }

      window.orderFrontRegardless();

      ThreadBound::new(window, disp)
    })?;

    Ok(Self { ns_window })
  }

  /// Moves the overlay to the given rect. Dispatches to main thread.
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let rect = rect.clone();
    self.ns_window.with(move |window| {
      // SAFETY: We are on the main thread (guaranteed by ThreadBound).
      let mtm = unsafe { MainThreadMarker::new_unchecked() };
      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint {
          x: f64::from(rect.x()),
          y: flipped_y(&rect, mtm),
        },
        objc2_foundation::NSSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );
      window.setFrame_display(ns_rect, true);
    })
  }

  /// Sets overlay opacity (0.0–1.0). For fade animations.
  pub fn set_opacity(&self, alpha: f32) -> crate::Result<()> {
    let alpha_f64 = f64::from(alpha);
    self.ns_window.with(move |window| {
      window.setAlphaValue(alpha_f64);
    })
  }

  /// Destroys the overlay window by ordering it out and dropping the
  /// handle.
  pub fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|window| {
      window.orderOut(None);
    })
  }
}

/// Converts a `Rect` top-left Y to the flipped `NSWindow` coordinate
/// system (origin at bottom-left of primary screen).
fn flipped_y(rect: &Rect, mtm: MainThreadMarker) -> f64 {
  let screen_height = NSScreen::screens(mtm)
    .into_iter()
    .next()
    .map_or(0.0, |s| s.frame().size.height);

  screen_height - f64::from(rect.y()) - f64::from(rect.height())
}

impl std::fmt::Debug for OverlayWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OverlayWindow").finish_non_exhaustive()
  }
}
