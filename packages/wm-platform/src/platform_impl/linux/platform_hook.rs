use std::{borrow::Cow, thread::JoinHandle};

use anyhow::bail;
use smithay::{
  desktop::{Space, Window},
  reexports::{
    pixman::Point,
    wayland_server::{Display, DisplayHandle},
  },
  utils::{Serial, SERIAL_COUNTER},
  wayland::seat::WaylandFocus,
};

use super::{
  event_loop::EventLoop, state::State, CalloopData, NativeWindow,
};

pub struct PlatformHook {
  event_loop: EventLoop,
}

impl PlatformHook {
  pub fn dedicated() -> anyhow::Result<Self> {
    let event_loop = EventLoop::new();

    Ok(Self { event_loop })
  }

  #[must_use]
  pub fn desktop_window(&self) -> NativeWindow {
    todo!()
  }

  #[must_use]
  pub fn is_foreground_window(&self, _: &NativeWindow) -> bool {
    false
  }

  pub fn mouse_position(&self) -> anyhow::Result<wm_common::Point> {
    todo!()
  }

  pub fn set_cursor_pos(&self, x: i32, y: i32) -> anyhow::Result<()> {
    self.event_loop.dispatch(move |data| {
      if let Some(pointer) = data.state.seat.get_pointer() {
        let point = smithay::utils::Point::new(f64::from(x), f64::from(y));
        let surface = data
          .state
          .space
          .element_under(point)
          .and_then(|(win, _point)| {
            win.wl_surface().map(std::borrow::Cow::into_owned)
          })
          .map(|surface| (surface, point));
        #[allow(clippy::cast_possible_truncation)]
        let event = smithay::input::pointer::MotionEvent {
          location: point,
          serial: SERIAL_COUNTER.next_serial(),
          time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32,
        };
        pointer.motion(&mut data.state, surface, &event);
      }
    })?;
    Ok(())
  }
}
