use wm_common::Point;

use crate::{platform_impl, Display, DisplayDevice, NativeWindow};

/// A thread-safe dispatcher for cross-platform operations.
///
/// The dispatcher provides synchronous methods for querying system state.
/// All operations are thread-safe and will automatically dispatch to the
/// appropriate thread when necessary.
///
/// # Thread safety
///
/// The dispatcher is cheap to clone and can be used from any thread.
/// Operations will automatically be dispatched to the main thread on
/// platforms that require it (such as macOS).
#[derive(Clone)]
pub struct Dispatcher {
  inner: platform_impl::EventLoopDispatcher,
}

impl Dispatcher {
  /// Creates a new dispatcher wrapping the platform-specific
  /// implementation.
  pub(crate) fn new(inner: platform_impl::EventLoopDispatcher) -> Self {
    Self { inner }
  }

  /// Gets a reference to the inner platform-specific dispatcher.
  pub(crate) fn inner(&self) -> &platform_impl::EventLoopDispatcher {
    &self.inner
  }

  /// Gets all active displays.
  ///
  /// Returns all displays that are currently active and available for use.
  pub fn displays(&self) -> crate::Result<Vec<Display>> {
    let inner_clone = self.inner.clone();
    self
      .inner
      .dispatch_sync(move || platform_impl::all_displays(&inner_clone))?
  }

  /// Gets all display devices.
  ///
  /// Returns all display devices including active, inactive, and
  /// disconnected ones.
  pub fn all_display_devices(&self) -> crate::Result<Vec<DisplayDevice>> {
    let inner_clone = self.inner.clone();
    self.inner.dispatch_sync(move || {
      platform_impl::all_display_devices(&inner_clone)
    })?
  }

  /// Gets the display containing the specified point.
  ///
  /// If no display contains the point, returns the primary display.
  pub fn display_from_point(
    &self,
    point: Point,
  ) -> crate::Result<Display> {
    let inner_clone = self.inner.clone();
    self.inner.dispatch_sync(move || {
      platform_impl::display_from_point(point, &inner_clone)
    })?
  }

  /// Gets the primary display.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Returns the display containing the menu bar.
  pub fn primary_display(&self) -> crate::Result<Display> {
    let inner_clone = self.inner.clone();
    self.inner.dispatch_sync(move || {
      platform_impl::primary_display(&inner_clone)
    })?
  }

  /// Gets all windows.
  ///
  /// Returns all windows that are currently on-screen and meet the
  /// filtering criteria, excluding system windows like Dock, menu bar,
  /// and desktop elements.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Uses `CGWindowListCopyWindowInfo` to enumerate windows
  ///   and filters out system applications and UI elements.
  pub fn all_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    let inner_clone = self.inner.clone();
    self
      .inner
      .dispatch_sync(move || platform_impl::all_windows(&inner_clone))?
  }

  /// Gets all windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for all windows
  /// from all running applications, including hidden applications.
  pub fn all_applications(&self) -> crate::Result<Vec<NativeWindow>> {
    let inner_clone = self.inner.clone();
    self.inner.dispatch_sync(move || {
      platform_impl::all_applications(&inner_clone)
    })?
  }

  /// Gets all visible windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for windows that are
  /// currently visible (not minimized or hidden).
  pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    let inner_clone = self.inner.clone();
    self.inner.dispatch_sync(move || {
      platform_impl::visible_windows(&inner_clone)
    })?
  }
}

// Thread safety: The underlying EventLoopDispatcher is already Send + Sync
unsafe impl Send for Dispatcher {}
unsafe impl Sync for Dispatcher {}
