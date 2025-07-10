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

use crate::state::Glaze;

impl SeatHandler for Glaze {
  type KeyboardFocus = WlSurface;
  type PointerFocus = WlSurface;
  type TouchFocus = WlSurface;

  fn seat_state(&mut self) -> &mut SeatState<Glaze> {
    &mut self.state.seat
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

delegate_seat!(Glaze);

//
// Wl Data Device
//

impl SelectionHandler for Glaze {
  type SelectionUserData = ();
}

impl DataDeviceHandler for Glaze {
  fn data_device_state(&self) -> &DataDeviceState {
    &self.state.data_device
  }
}

impl ClientDndGrabHandler for Glaze {}
impl ServerDndGrabHandler for Glaze {}

delegate_data_device!(Glaze);

//
// Wl Output & Xdg Output
//

impl OutputHandler for Glaze {}
delegate_output!(Glaze);
