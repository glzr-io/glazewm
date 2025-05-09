use anyhow::Context;
use wm_common::{
  LengthValue, Rect, RectDelta, TilingDirection, WindowState,
};
use wm_platform::NativeWindow;

use crate::{
  commands::container::{attach_container, detach_container},
  models::{
    Container, SplitContainer, TilingWindow, WindowContainer, Workspace,
  },
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_grid_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<WindowContainer> {
  let new_window = TilingWindow::new(
    None,
    native_window,
    None,
    RectDelta::new(
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
    ),
    Rect {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    },
    false,
    config.value.gaps.clone(),
    Vec::new(),
    None,
  );

  // Handle empty workspace
  if target_workspace.child_count() == 0 {
    attach_container(
      &new_window.clone().into(),
      &target_workspace.clone().into(),
      Some(0),
    )?;
    return new_window.as_window_container();
  }

  // Initialize grid for first two windows
  // TODO no need for the extra top level container
  let first_child = {
    let children = target_workspace.borrow_children();
    children.get(0).unwrap().clone()
  };

  if first_child.as_window_container().is_ok()
    || (target_workspace.child_count() == 1
      && first_child.as_direction_container().is_err())
  {
    // Create main vertical container for rows
    let main_container = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );

    // Create first row
    let first_row = SplitContainer::new(
      TilingDirection::Horizontal,
      config.value.gaps.clone(),
    );

    // Detach the existing window
    let existing_window = detach_container(first_child, true)?;

    // Set up structure
    attach_container(
      &main_container.clone().into(),
      &target_workspace.clone().into(),
      Some(0),
    )?;

    attach_container(
      &first_row.clone().into(),
      &main_container.clone().into(),
      Some(0),
    )?;

    // Attach windows
    attach_container(
      &existing_window,
      &first_row.clone().into(),
      Some(0),
    )?;
    attach_container(
      &new_window.clone().into(),
      &first_row.clone().into(),
      Some(1),
    )?;

    return new_window.as_window_container();
  }

  // Get main container (rows container)
  let main_container = first_child;

  // Calculate total windows and target columns
  let mut total_windows = 1; // Start with 1 for the new window

  for i in 0..main_container.child_count() {
    let row_ref = main_container.borrow_children();
    let row = row_ref.get(i).unwrap().clone();
    total_windows += row.child_count();
  }

  let (_, target_cols) = calculate_grid_dimensions(total_windows);

  // Find target window position
  let (target_row, target_col) =
    find_window_position(&main_container, &target_container)?;

  // Get current row
  let current_row = {
    let rows = main_container.borrow_children();
    rows.get(target_row).unwrap().clone()
  };

  // If current row has space to the right, insert there
  if current_row.child_count() < target_cols {
    attach_container(
      &new_window.clone().into(),
      &current_row,
      Some(target_col + 1),
    )?;
    return new_window.as_window_container();
  }

  // Check if target is at end of row
  let is_end_of_row = target_col == current_row.child_count() - 1;

  if is_end_of_row {
    // For end-of-row targets:
    // First try to place in the same column in the row above
    if target_row > 0 {
      let above_row = {
        let rows = main_container.borrow_children();
        rows.get(target_row - 1).unwrap().clone()
      };

      if above_row.child_count() <= target_col {
        // Position is free, place window
        attach_container(
          &new_window.clone().into(),
          &above_row,
          Some(target_col),
        )?;
        return new_window.as_window_container();
      }
    }

    // Next try to place in the same column in the row below
    if target_row + 1 < main_container.child_count() {
      let below_row = {
        let rows = main_container.borrow_children();
        rows.get(target_row + 1).unwrap().clone()
      };

      if below_row.child_count() <= target_col {
        // Position is free, place window
        attach_container(
          &new_window.clone().into(),
          &below_row,
          Some(target_col),
        )?;
        return new_window.as_window_container();
      } else {
        // Position exists but is occupied, need to cascade windows
        // Starting from this position in the row below
        return cascade_from_position(
          new_window,
          main_container,
          target_row + 1,
          target_col,
          target_cols,
          config,
        );
      }
    }

    // Need to create a new row or find another spot

    // Check if any row has space at the same column
    for i in 0..main_container.child_count() {
      if i != target_row {
        let row = {
          let rows = main_container.borrow_children();
          rows.get(i).unwrap().clone()
        };

        if row.child_count() <= target_col {
          // Row has space at the target column
          attach_container(
            &new_window.clone().into(),
            &row,
            Some(target_col),
          )?;
          return new_window.as_window_container();
        }
      }
    }

    // No existing row has space at the target column
    // Create a new row if all rows are full, otherwise use any row with
    // space

    let mut any_row_has_space = false;
    let mut space_row_idx = 0;

    for i in 0..main_container.child_count() {
      let row = {
        let rows = main_container.borrow_children();
        rows.get(i).unwrap().clone()
      };

      if row.child_count() < target_cols {
        any_row_has_space = true;
        space_row_idx = i;
        break;
      }
    }

    if any_row_has_space {
      // Use row with space
      let row = {
        let rows = main_container.borrow_children();
        rows.get(space_row_idx).unwrap().clone()
      };

      attach_container(
        &new_window.clone().into(),
        &row,
        None, // Add at the end
      )?;
    } else {
      // Create a new row
      let new_row = SplitContainer::new(
        TilingDirection::Horizontal,
        config.value.gaps.clone(),
      );

      attach_container(
        &new_row.clone().into(),
        &main_container,
        Some(target_row + 1), // Insert after target row
      )?;

      attach_container(
        &new_window.clone().into(),
        &new_row.clone().into(),
        Some(target_col), // Same column as target
      )?;
    }
  } else {
    // Target is not at end of row
    // Cascade windows starting from the window right of the target
    return cascade_from_position(
      new_window,
      main_container,
      target_row,
      target_col + 1,
      target_cols,
      config,
    );
  }

  new_window.as_window_container()
}

// Helper function to cascade windows starting from a specific position
fn cascade_from_position(
  new_window: TilingWindow,
  main_container: Container,
  start_row: usize,
  start_col: usize,
  target_cols: usize,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  // Get the window at the start position
  let start_row_container = {
    let rows = main_container.borrow_children();
    rows.get(start_row).unwrap().clone()
  };

  // Get the window to shift
  let window_to_shift = {
    let children = start_row_container.borrow_children();
    children.get(start_col).unwrap().clone()
  };

  // Detach the window
  let shifted = detach_container(window_to_shift, true)?;

  // Place new window
  attach_container(
    &new_window.clone().into(),
    &start_row_container,
    Some(start_col),
  )?;

  // Start cascading from next position
  let mut current_window = shifted;
  let mut curr_row = start_row;
  let mut curr_col = start_col + 1;

  // If curr_col is beyond the row width, move to the next row
  if curr_col >= target_cols {
    curr_row += 1;
    curr_col = 0;
  }

  loop {
    // Get or create current row
    let current_row_container = if curr_row < main_container.child_count()
    {
      let rows = main_container.borrow_children();
      rows.get(curr_row).unwrap().clone()
    } else {
      // Before creating a new row, check if any existing row has space
      let mut row_with_space = None;

      for i in 0..main_container.child_count() {
        let row = {
          let rows = main_container.borrow_children();
          rows.get(i).unwrap().clone()
        };

        if row.child_count() < target_cols {
          row_with_space = Some((i, row.child_count()));
          break;
        }
      }

      if let Some((row_idx, col_idx)) = row_with_space {
        // Found a row with space
        curr_row = row_idx;
        curr_col = col_idx;

        let rows = main_container.borrow_children();
        rows.get(curr_row).unwrap().clone()
      } else {
        // Create a new row
        let new_row = SplitContainer::new(
          TilingDirection::Horizontal,
          config.value.gaps.clone(),
        );

        attach_container(
          &new_row.clone().into(),
          &main_container,
          None, // Append at the end
        )?;

        curr_col = 0; // Start at beginning of the new row
        new_row.clone().into()
      }
    };

    // Check if position is occupied
    if curr_col < current_row_container.child_count() {
      // Position is occupied, shift that window
      let window_at_pos = {
        let children = current_row_container.borrow_children();
        children.get(curr_col).unwrap().clone()
      };

      let next_shifted = detach_container(window_at_pos, true)?;

      // Place our current window
      attach_container(
        &current_window,
        &current_row_container,
        Some(curr_col),
      )?;

      // Continue with the next shifted window
      current_window = next_shifted;
      curr_col += 1;

      // If we've reached the end of the row, move to next row
      if curr_col >= target_cols {
        curr_row += 1;
        curr_col = 0;
      }
    } else {
      // Position is free, place window and we're done
      attach_container(
        &current_window,
        &current_row_container,
        Some(curr_col),
      )?;
      break;
    }
  }

  new_window.as_window_container()
}

// Find a window's position in the grid
fn find_window_position(
  main_container: &Container,
  target_container: &Container,
) -> anyhow::Result<(usize, usize)> {
  for i in 0..main_container.child_count() {
    let row_ref = main_container.borrow_children();
    let row = row_ref.get(i).unwrap().clone();

    for j in 0..row.child_count() {
      let child_ref = row.borrow_children();
      let child = child_ref.get(j).unwrap().clone();

      if child.id() == target_container.id() {
        return Ok((i, j));
      }
    }
  }

  Err(anyhow::anyhow!("Target container not found"))
}

// Calculate the optimal grid dimensions based on the number of windows
// Prefer more columns than rows when uneven
fn calculate_grid_dimensions(total_windows: usize) -> (usize, usize) {
  let sqrt = (total_windows as f64).sqrt().ceil() as usize;

  if sqrt * (sqrt - 1) >= total_windows {
    // We can use one fewer row
    (sqrt - 1, sqrt)
  } else {
    // Square grid
    (sqrt, sqrt)
  }
}
