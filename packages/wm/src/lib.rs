#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

pub mod app_command;
pub mod cleanup;
pub mod common;
pub mod containers;
pub mod ipc_client;
pub mod ipc_server;
pub mod monitors;
pub mod sys_tray;
pub mod user_config;
pub mod windows;
pub mod wm;
pub mod wm_event;
pub mod wm_state;
pub mod workspaces;
