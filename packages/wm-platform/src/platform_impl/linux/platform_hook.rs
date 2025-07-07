use std::thread::JoinHandle;

use anyhow::bail;
use smithay::reexports::{
  calloop::EventLoop,
  wayland_server::{Display, DisplayHandle},
};

use super::{state::State, CalloopData};

pub struct PlatformHook<'a> {
  event_loop: EventLoop<'a, CalloopData>,
}

impl<'a> PlatformHook<'a> {
  pub fn dedicated() -> anyhow::Result<Self> {
    let mut event_loop = EventLoop::<CalloopData>::try_new()?;

    let display: Display<State> = Display::new()?;
    let handle = display.handle();
    let state = State::new(&mut event_loop, display);

    let mut data = CalloopData {
      state,
      display_handle: handle,
    };

    if let Err(e) = super::winit::init_winit(&mut event_loop, &mut data) {
      bail!("Failed to initialize winit: {}", e);
    }

    Ok(Self { event_loop })
  }
}
