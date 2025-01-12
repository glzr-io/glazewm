use crate::models::Container;

#[derive(Debug, Clone)]
pub struct InsertionTarget {
  pub target_parent: Container,
  pub target_index: usize,
  pub prev_tiling_size: f32,
  pub prev_sibling_count: usize,
}
