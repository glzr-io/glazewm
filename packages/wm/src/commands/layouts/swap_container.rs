use crate::{models::Container, traits::CommonGetters};

pub fn swap_container(
  container_a: &mut Container,
  container_b: &mut Container,
) {
  let parent_a = container_a.parent().expect("Container A has no parent");
  let parent_b = container_b.parent().expect("Container B has no parent");

  let index_a = container_a.index();
  let index_b = container_b.index();

  // Swap the containers in each parent's children array
  let mut children_a = parent_a.borrow_children_mut();
  let mut children_b = parent_b.borrow_children_mut();
  children_a.remove(index_a);
  children_a.insert(index_a, container_b.clone());
  children_b.remove(index_b);
  children_b.insert(index_b, container_a.clone());

  // Update the parent references
  *container_a.borrow_parent_mut() = Some(parent_b.clone());
  *container_b.borrow_parent_mut() = Some(parent_a.clone());
}
