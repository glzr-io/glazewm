pub trait TilingContainer {
  fn is_tiling_active(&self) -> bool;
  fn size_percentage(&self) -> f32;
}
