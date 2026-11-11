use crate::{platform_impl, Color, Dispatcher, Rect, WindowId};

/// Identifier for a [`TabStrip`].
///
/// Used to match a [`TabStripEvent`] to the strip that emitted it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TabStripId(pub(crate) isize);

/// Visual state of a [`TabStrip`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabStripState {
  /// Bounds of the strip.
  pub rect: Rect,

  /// Number of tabs to draw. Each tab is labeled with its 1-based index.
  pub tab_count: usize,

  /// 0-based index of the tab to highlight as active.
  pub active_index: usize,

  /// Background color of the active tab.
  pub active_color: Color,

  /// Background color of the inactive tabs.
  pub inactive_color: Color,

  /// Color of the tab labels.
  pub text_color: Color,

  /// Whether to draw the 1-based index label on each tab.
  pub show_labels: bool,

  /// Radius in pixels of the strip's rounded bottom corners. The top
  /// corners are always square. `0` gives square bottom corners too.
  pub corner_radius: i32,

  /// Window to place the strip directly above in the z-order. If `None`,
  /// the strip is placed at the top of the non-topmost windows.
  pub z_order_after: Option<WindowId>,

  /// Whether the strip is shown.
  pub is_visible: bool,
}

/// A non-activatable strip of numbered tabs, shown above a group's
/// active page.
///
/// The strip is not managed as a window and does not appear in the
/// taskbar or task switcher.
///
/// # Platform-specific
///
/// - **macOS**: Not yet implemented. All methods are no-ops.
pub struct TabStrip {
  inner: platform_impl::TabStrip,
}

impl TabStrip {
  /// Creates a new hidden tab strip.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let inner = platform_impl::TabStrip::new(dispatcher)?;
    Ok(Self { inner })
  }

  /// Gets the identifier of the tab strip.
  #[must_use]
  pub fn id(&self) -> TabStripId {
    self.inner.id()
  }

  /// Applies the given state to the tab strip.
  ///
  /// Does nothing if the state is unchanged since the last call.
  pub fn set_state(&mut self, state: &TabStripState) -> crate::Result<()> {
    self.inner.set_state(state)
  }
}

/// An event emitted by a [`TabStrip`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabStripEvent {
  /// A tab in a strip was clicked.
  TabClicked {
    /// ID of the strip that was clicked.
    strip_id: TabStripId,

    /// 0-based index of the clicked tab.
    index: usize,
  },
}
