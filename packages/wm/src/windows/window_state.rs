#[derive(Clone, Debug)]
pub enum WindowState {
  Floating,
  Fullscreen,
  Maximized,
  Minimized,
  Tiling,
}
