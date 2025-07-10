use smithay::{
  backend::renderer::utils::on_commit_buffer_handler,
  delegate_compositor, delegate_shm,
  reexports::wayland_server::{
    protocol::{wl_buffer, wl_surface::WlSurface},
    Client,
  },
  wayland::{
    buffer::BufferHandler,
    compositor::{
      get_parent, is_sync_subsurface, CompositorClientState,
      CompositorHandler, CompositorState,
    },
    shm::{ShmHandler, ShmState},
  },
};

use super::xdg_shell;
use crate::{
  grabs::resize_grab,
  state::{ClientState, Glaze},
};

impl CompositorHandler for Glaze {
  fn compositor_state(&mut self) -> &mut CompositorState {
    &mut self.state.compositor
  }

  fn client_compositor_state<'a>(
    &self,
    client: &'a Client,
  ) -> &'a CompositorClientState {
    &client.get_data::<ClientState>().unwrap().compositor_state
  }

  fn commit(&mut self, surface: &WlSurface) {
    on_commit_buffer_handler::<Self>(surface);
    if !is_sync_subsurface(surface) {
      let mut root = surface.clone();
      while let Some(parent) = get_parent(&root) {
        root = parent;
      }
      if let Some(window) = self
        .space
        .elements()
        .find(|w| w.toplevel().unwrap().wl_surface() == &root)
      {
        window.on_commit();
      }
    }

    xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
    resize_grab::handle_commit(&mut self.space, surface);
  }
}

impl BufferHandler for Glaze {
  fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for Glaze {
  fn shm_state(&self) -> &ShmState {
    &self.state.shm
  }
}

delegate_compositor!(Glaze);
delegate_shm!(Glaze);
