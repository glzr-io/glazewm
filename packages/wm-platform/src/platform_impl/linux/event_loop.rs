use anyhow::bail;
use smithay::reexports::{calloop, wayland_server::Display};
use wm_common::ParsedConfig;

use super::{state::Glaze, CalloopData};

pub type DispatchFn = Box<Box<dyn FnOnce(&mut CalloopData) + Send>>;

#[derive(Clone, Copy)]
struct RawDispatchFn(
  pub *mut Box<dyn FnOnce(&mut CalloopData) + Send + 'static>,
);

unsafe impl Send for RawDispatchFn {}

pub struct EventLoop {
  join_handle: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
  dispatcher: calloop::channel::Sender<RawDispatchFn>,
}

impl EventLoop {
  pub fn new(config: &ParsedConfig) -> Self {
    let (dispatcher, receiver) = calloop::channel::channel();

    let config = config.clone();

    let join_handle = std::thread::spawn(move || {
      let mut event_loop = calloop::EventLoop::<CalloopData>::try_new()?;

      let display: Display<Glaze> = Display::new()?;
      let handle = display.handle();
      let state = Glaze::new(&mut event_loop, display, config);

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
        |event: calloop::channel::Event<RawDispatchFn>, (), data| {
          match event {
            calloop::channel::Event::Msg(raw) => {
              let func: DispatchFn = unsafe { Box::from_raw(raw.0) };
              // Execute the function in the main thread
              func(data);
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

  pub fn dispatch<F>(&self, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce(&mut CalloopData) + Send + 'static,
  {
    let cb = Box::new(Box::new(callback)
      as Box<dyn FnOnce(&mut CalloopData) + Send + 'static>);
    let raw = RawDispatchFn(Box::into_raw(cb));
    if let Err(e) = self.dispatcher.send(raw) {
      unsafe {
        _ = Box::from_raw(raw.0);
      } // Ensure we clean up the memory if sending fails
      return Err(anyhow::anyhow!(
        "Failed to send dispatch callback: {}",
        e
      ));
    }

    Ok(())
  }

  pub fn dispatch_and_wait<F, R>(&self, callback: F) -> anyhow::Result<R>
  where
    F: FnOnce(&mut CalloopData) -> R + Send + 'static,
    R: Send + 'static,
  {
    let (tx, rx) = tokio::sync::oneshot::channel();

    self.dispatch(move |state| {
      let res = callback(state);

      if tx.send(res).is_err() {
        tracing::error!(
          "Failed to send result from callback, receiver dropped"
        );
      }
    });

    let res = rx.blocking_recv()?;

    Ok(res)
  }
}
