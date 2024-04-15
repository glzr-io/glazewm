extern crate winapi;

use winapi::um::winuser::SetCursorPos;
use anyhow::{Result, Error};
use crate::containers::{Container, SplitContainer};
use crate::containers::traits::PositionGetters;

pub fn center_cursor_on_container(
    target: &Container 
) -> Result<(), Error> {
    // do i pull from user config here or is that handled elsewhere?
    let is_enabled = true;

    // target is currently missing is_detached and focus_index 
    // if !is_enabled || target.is_detached() || target.focus_index < 0 {
    if !is_enabled {
        return Ok(());
    }

    // Calculate center point of focused window.
    let center_x = target.x()? + (target.width()? / 2);
    let center_y = target.y()? + (target.height()? / 2);

    unsafe {
        SetCursorPos(center_x, center_y);
    }
    
    Ok(())
}