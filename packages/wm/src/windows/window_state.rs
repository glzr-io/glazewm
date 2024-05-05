/// Represents the various states a window can be in.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowState {
  Floating,
  Fullscreen,
  Minimized,
  Tiling,
}
