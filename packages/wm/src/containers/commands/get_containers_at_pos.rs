use crate::{
  common::Point,
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, WindowContainer,
  },
  wm_state::WmState,
};

/// Return all window under the mouse position 
pub fn get_containers_at_position(
  state: &WmState,
  position: &Point,
) -> Vec<WindowContainer> {
  state
    .root_container
    .descendants()
    .filter_map(|container| match container {
      Container::TilingWindow(tiling) => {
        Some(WindowContainer::TilingWindow(tiling))
      }
      Container::NonTilingWindow(non_tiling) => {
        Some(WindowContainer::NonTilingWindow(non_tiling))
      }
      _ => None,
    })
    .filter(|c| {
      let frame = c.to_rect();
      frame.unwrap().contains_point(&position)
    })
    .collect()
}
