#![feature(iterator_try_collect)]

pub mod commands;
pub mod events;
pub mod ipc_server;
pub mod models;
pub mod pending_sync;
pub mod sys_tray;
pub mod traits;
pub mod user_config;
pub mod wm;
pub mod wm_state;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod test;
