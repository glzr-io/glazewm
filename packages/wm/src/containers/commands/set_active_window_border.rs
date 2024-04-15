extern crate winapi;

use std::convert::TryInto;
use std::ptr;
use winapi::um::dwmapi::DwmSetWindowAttribute;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::PVOID;
use anyhow::{Result, Error};
use crate::containers::Container;

pub fn handle_set_active_window_border(
    target: Container
) -> Result<(), Error> {

    Ok(())
}