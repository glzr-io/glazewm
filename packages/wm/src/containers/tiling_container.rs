pub trait TilingContainer {
  fn can_tile(&self) -> bool;
  fn size_percentage(&self) -> f32;
}
