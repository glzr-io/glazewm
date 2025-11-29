use super::flatten_split_container;
use crate::{
    models::Container,
    traits::{CommonGetters, TilingSizeGetters, MIN_TILING_SIZE},
};

/// Removes a container from the tree.
///
/// **Hybrid Cleanup Strategy:**
/// - If removing a **Split Container** (Parent): We flatten the tree to preventing nesting issues.
/// - If removing a **Window** (Child): We SKIP flattening to prevent "No Ancestor" crashes.
///   We rely on `rebuild_spiral_layout` to clean up the tree structure.
#[allow(clippy::needless_pass_by_value)]
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
    let parent = match child_to_remove.parent() {
        Some(parent) => parent,
        None => return Ok(()),
    };

    // Capture the type BEFORE we detach anything
    let is_split_container = child_to_remove.is_split();

    // 1. Resize siblings
    if let Ok(child_tiling) = child_to_remove.as_tiling_container() {
        let tiling_siblings: Vec<_> = parent
            .tiling_children()
            .filter(|c| c.id() != child_to_remove.id())
            .collect();

        let available_size = tiling_siblings.iter().fold(0.0, |sum, container| {
            sum + container.tiling_size() - MIN_TILING_SIZE
        });

        if !tiling_siblings.is_empty() && available_size > 0.001 {
            for sibling in &tiling_siblings {
                let resize_factor = (sibling.tiling_size() - MIN_TILING_SIZE) / available_size;
                let size_delta = resize_factor * child_tiling.tiling_size();
                sibling.set_tiling_size(sibling.tiling_size() + size_delta);
            }
        } else if !tiling_siblings.is_empty() {
             let chunk = child_tiling.tiling_size() / tiling_siblings.len() as f32;
             for sibling in &tiling_siblings {
                 sibling.set_tiling_size(sibling.tiling_size() + chunk);
             }
        }
    }

    // 2. Remove the child
    parent
        .borrow_children_mut()
        .retain(|c| c.id() != child_to_remove.id());

    parent
        .borrow_child_focus_order_mut()
        .retain(|id| *id != child_to_remove.id());

    *child_to_remove.borrow_parent_mut() = None;

    // 3. Conditional Flattening
    if let Some(split_parent) = parent.as_split().cloned() {
        if split_parent.parent().is_some() && split_parent.child_count() == 1 {
            // FIX: Only flatten if we just moved a Split Container.
            if is_split_container {
                // This call is now safe because we fixed flatten_split_container above
                flatten_split_container(split_parent)?;
            }
        }
    }

    Ok(())
}
