use smithay::utils::SERIAL_COUNTER;
use wm_common::ParsedConfig;

use super::{event_loop::EventLoop, NativeWindow};

pub struct PlatformHook {
  event_loop: EventLoop,
}

impl PlatformHook {
  pub fn dedicated(config: &ParsedConfig) -> anyhow::Result<Self> {
    let event_loop = EventLoop::new(config);

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
        let surface = data.state.surface_under(point);
        #[allow(clippy::cast_possible_truncation)]
        let event = smithay::input::pointer::MotionEvent {
          location: point,
          serial: SERIAL_COUNTER.next_serial(),
          time: data.state.clock.now().as_millis(),
        };
        pointer.motion(&mut data.state, surface, &event);
      }
    })?;
    Ok(())
  }
}
