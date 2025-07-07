use std::thread::JoinHandle;

use anyhow::bail;
use smithay::reexports::{
  calloop,
  wayland_server::{Display, DisplayHandle},
};

use super::{state::State, CalloopData};

pub struct EventLoop {
  join_handle: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
  dispatcher: calloop::channel::Sender<Box<dyn FnOnce() + Send>>,
}

impl EventLoop {
  pub fn new() -> Self {
    let (dispatcher, receiver) = calloop::channel::channel();

    let join_handle = std::thread::spawn(move || {
      let mut event_loop = calloop::EventLoop::<CalloopData>::try_new()?;

      let display: Display<State> = Display::new()?;
      let handle = display.handle();
      let state = State::new(&mut event_loop, display);

      let mut data = CalloopData {
        state,
        display_handle: handle,
      };

      if let Err(e) = super::winit::init_winit(&mut event_loop, &mut data)
      {
        bail!("Failed to initialize winit: {}", e);
      }

      let _token = match event_loop.handle().insert_source(
        receiver,
        |event: calloop::channel::Event<Box<dyn FnOnce() + Send>>,
         (),
         _| {
          match event {
            calloop::channel::Event::Msg(func) => {
              // Execute the function in the main thread
              func();
            }
            calloop::channel::Event::Closed => {}
          }
        },
      ) {
        Ok(token) => token,
        Err(e) => bail!("Failed to insert channel source: {}", e),
      };

      event_loop.run(None, &mut data, |data| {})?;

      Ok(())
    });

    Self {
      join_handle: Some(join_handle),
      dispatcher,
    }
  }
}
