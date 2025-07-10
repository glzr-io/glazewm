use smithay::{
  delegate_xdg_shell,
  desktop::{
    find_popup_root_surface, get_popup_toplevel_coords, PopupKind,
    PopupManager, Space, Window,
  },
  input::{
    pointer::{Focus, GrabStartData as PointerGrabStartData},
    Seat,
  },
  reexports::{
    wayland_protocols::xdg::shell::server::xdg_toplevel,
    wayland_server::{
      protocol::{wl_seat, wl_surface::WlSurface},
      Resource,
    },
  },
  utils::{Rectangle, Serial},
  wayland::{
    compositor::with_states,
    shell::xdg::{
      PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler,
      XdgShellState, XdgToplevelSurfaceData,
    },
  },
};
use wm_common::{InitialWindowState, WindowState};

use crate::{
  grabs::{MoveSurfaceGrab, ResizeSurfaceGrab},
  state::Glaze,
  NativeWindow,
};

impl XdgShellHandler for Glaze {
  fn xdg_shell_state(&mut self) -> &mut XdgShellState {
    &mut self.state.xdg_shell
  }

  // Called whenever a new window is added to the compositor
  fn new_toplevel(&mut self, surface: ToplevelSurface) {
    let window = Window::new_wayland_window(surface);

    let native_window = NativeWindow::new(window);
    self.windows.new_window(native_window);
  }

  /// Called whenever a window is closed
  fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
    self.windows.window_close(&surface);
  }

  fn new_popup(
    &mut self,
    surface: PopupSurface,
    _positioner: PositionerState,
  ) {
    self.unconstrain_popup(&surface);
    let _ = self.popups.track_popup(PopupKind::Xdg(surface));
  }

  fn reposition_request(
    &mut self,
    surface: PopupSurface,
    positioner: PositionerState,
    token: u32,
  ) {
    surface.with_pending_state(|state| {
      let geometry = positioner.get_geometry();
      state.geometry = geometry;
      state.positioner = positioner;
    });
    self.unconstrain_popup(&surface);
    surface.send_repositioned(token);
  }

  fn move_request(
    &mut self,
    surface: ToplevelSurface,
    seat: wl_seat::WlSeat,
    serial: Serial,
  ) {
    let seat = Seat::from_resource(&seat).unwrap();

    let wl_surface = surface.wl_surface();

    if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
      let pointer = seat.get_pointer().unwrap();

      let window = self
        .space
        .elements()
        .find(|w| w.toplevel().unwrap().wl_surface() == wl_surface)
        .unwrap()
        .clone();
      let initial_window_location =
        self.space.element_location(&window).unwrap();

      let grab = MoveSurfaceGrab {
        start_data,
        window,
        initial_window_location,
      };

      pointer.set_grab(self, grab, serial, Focus::Clear);
    }
  }

  fn resize_request(
    &mut self,
    surface: ToplevelSurface,
    seat: wl_seat::WlSeat,
    serial: Serial,
    edges: xdg_toplevel::ResizeEdge,
  ) {
    let seat = Seat::from_resource(&seat).unwrap();

    let wl_surface = surface.wl_surface();

    if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
      let pointer = seat.get_pointer().unwrap();

      let window = self
        .space
        .elements()
        .find(|w| w.toplevel().unwrap().wl_surface() == wl_surface)
        .unwrap()
        .clone();
      let initial_window_location =
        self.space.element_location(&window).unwrap();
      let initial_window_size = window.geometry().size;

      surface.with_pending_state(|state| {
        state.states.set(xdg_toplevel::State::Resizing);
      });

      surface.send_pending_configure();

      let grab = ResizeSurfaceGrab::start(
        start_data,
        window,
        edges.into(),
        Rectangle::new(initial_window_location, initial_window_size),
      );

      pointer.set_grab(self, grab, serial, Focus::Clear);
    }
  }

  fn grab(
    &mut self,
    _surface: PopupSurface,
    _seat: wl_seat::WlSeat,
    _serial: Serial,
  ) {
    // TODO popup grabs
  }
}

// Xdg Shell
delegate_xdg_shell!(Glaze);

fn check_grab(
  seat: &Seat<Glaze>,
  surface: &WlSurface,
  serial: Serial,
) -> Option<PointerGrabStartData<Glaze>> {
  let pointer = seat.get_pointer()?;

  // Check that this surface has a click grab.
  if !pointer.has_grab(serial) {
    return None;
  }

  let start_data = pointer.grab_start_data()?;

  let (focus, _) = start_data.focus.as_ref()?;
  // If the focus was for a different surface, ignore the request.
  if !focus.id().same_client_as(&surface.id()) {
    return None;
  }

  Some(start_data)
}

/// Should be called on `WlSurface::commit`
pub fn handle_commit(
  popups: &mut PopupManager,
  space: &Space<Window>,
  surface: &WlSurface,
) {
  // Handle toplevel commits.
  if let Some(window) = space
    .elements()
    .find(|w| w.toplevel().unwrap().wl_surface() == surface)
    .cloned()
  {
    let initial_configure_sent = with_states(surface, |states| {
      states
        .data_map
        .get::<XdgToplevelSurfaceData>()
        .unwrap()
        .lock()
        .unwrap()
        .initial_configure_sent
    });

    if !initial_configure_sent {
      window.toplevel().unwrap().send_configure();
    }
  }

  // Handle popup commits.
  popups.commit(surface);
  if let Some(popup) = popups.find_popup(surface) {
    match popup {
      PopupKind::Xdg(ref xdg) => {
        if !xdg.is_initial_configure_sent() {
          // NOTE: This should never fail as the initial configure is
          // always allowed.
          xdg.send_configure().expect("initial configure failed");
        }
      }
      PopupKind::InputMethod(ref _input_method) => {}
    }
  }
}

impl Glaze {
  fn unconstrain_popup(&self, popup: &PopupSurface) {
    let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone()))
    else {
      return;
    };
    let Some(window) = self
      .space
      .elements()
      .find(|w| w.toplevel().unwrap().wl_surface() == &root)
    else {
      return;
    };

    let output = self.space.outputs().next().unwrap();
    let output_geo = self.space.output_geometry(output).unwrap();
    let window_geo = self.space.element_geometry(window).unwrap();

    // The target geometry for the positioner should be relative to its
    // parent's geometry, so we will compute that here.
    let mut target = output_geo;
    target.loc -=
      get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
    target.loc -= window_geo.loc;

    popup.with_pending_state(|state| {
      state.geometry = state.positioner.get_unconstrained_geometry(target);
    });
  }
}
