mod compositor;
mod xdg_shell;

//
// Wl Seat
use smithay::{
  delegate_data_device, delegate_output, delegate_seat,
  input::{Seat, SeatHandler, SeatState},
  reexports::wayland_server::{protocol::wl_surface::WlSurface, Resource},
  wayland::{
    output::OutputHandler,
    selection::{
      data_device::{
        set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler,
        DataDeviceState, ServerDndGrabHandler,
      },
      SelectionHandler,
    },
  },
};

use crate::state::State;

impl SeatHandler for State {
  type KeyboardFocus = WlSurface;
  type PointerFocus = WlSurface;
  type TouchFocus = WlSurface;

  fn seat_state(&mut self) -> &mut SeatState<State> {
    &mut self.seat_state
  }

  fn cursor_image(
    &mut self,
    _seat: &Seat<Self>,
    _image: smithay::input::pointer::CursorImageStatus,
  ) {
  }

  fn focus_changed(
    &mut self,
    seat: &Seat<Self>,
    focused: Option<&WlSurface>,
  ) {
    let dh = &self.display_handle;
    let client = focused.and_then(|s| dh.get_client(s.id()).ok());
    set_data_device_focus(dh, seat, client);
  }
}

delegate_seat!(State);

//
// Wl Data Device
//

impl SelectionHandler for State {
  type SelectionUserData = ();
}

impl DataDeviceHandler for State {
  fn data_device_state(&self) -> &DataDeviceState {
    &self.data_device_state
  }
}

impl ClientDndGrabHandler for State {}
impl ServerDndGrabHandler for State {}

delegate_data_device!(State);

//
// Wl Output & Xdg Output
//

impl OutputHandler for State {}
delegate_output!(State);
