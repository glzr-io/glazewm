use wm_common::{BindingModeConfig, KeybindingConfig};

use super::MouseHook;
use crate::{
  platform_impl::{EventLoop, WindowEventHook},
  DisplayHook, KeyboardHook, WindowEventType,
};

pub struct PlatformHookInstaller {}

pub struct PlatformHook {
  event_loop: Option<EventLoop>,
}
impl PlatformHook {
  pub fn dedicated() -> anyhow::Result<Self> {
    Ok(Self { event_loop: None })
  }

  pub fn remote() -> anyhow::Result<(Self, PlatformHookInstaller)> {
    todo!("Implement remote platform hook installation");
  }

  pub async fn create_mouse_listener(
    &mut self,
  ) -> anyhow::Result<MouseHook> {
    if self.event_loop.is_none() {
      self.event_loop = Some(EventLoop::new()?);
    }
    // Safety: The event loop is guaranteed to be initialized before this
    // method is called.
    let event_loop =
      unsafe { self.event_loop.as_mut().unwrap_unchecked() };
    let (mouse_hook, installer) =
      MouseHook::new(event_loop.message_window_handle());

    event_loop.install("Mouse Hook Install", installer).await?;

    Ok(mouse_hook)
  }

  pub async fn create_display_listener(
    &mut self,
  ) -> anyhow::Result<DisplayHook> {
    let (hook, installer) = DisplayHook::new();

    if self.event_loop.is_none() {
      self.event_loop = Some(EventLoop::new()?);
    }
    // Safety: The event loop is guaranteed to be initialized before this
    // method is called.
    let event_loop =
      unsafe { self.event_loop.as_mut().unwrap_unchecked() };
    event_loop
      .install("Display Hook Install", installer)
      .await?;
    Ok(hook)
  }

  pub async fn with_window_events(
    &mut self,
    events: &'static [WindowEventType],
  ) -> anyhow::Result<WindowEventHook> {
    let (window_event_hook, installer) = WindowEventHook::new(events);

    if self.event_loop.is_none() {
      self.event_loop = Some(EventLoop::new()?);
    }
    // Safety: The event loop is guaranteed to be initialized before this
    let event_loop =
      unsafe { self.event_loop.as_mut().unwrap_unchecked() };
    tracing::warn!(
      "Installing Window Event Hook for events: {:?}",
      events
    );
    event_loop
      .install("Window Event Install", installer)
      .await?;
    tracing::warn!("Installed Window Event Hook for events: {:?}", events);
    Ok(window_event_hook)
  }

  pub async fn create_keyboard_listener(
    &mut self,
    config: &[KeybindingConfig],
  ) -> anyhow::Result<KeyboardHook> {
    if self.event_loop.is_none() {
      self.event_loop = Some(EventLoop::new()?);
    }
    // Safety: The event loop is guaranteed to be initialized before this
    let event_loop =
      unsafe { self.event_loop.as_mut().unwrap_unchecked() };
    let (hook, installer) = KeyboardHook::new(config);

    tracing::warn!("Installing Keyboard hook");
    event_loop
      .install("Keyboard Hook Install", installer)
      .await?;

    Ok(hook)
  }

  pub fn update_keybinds(
    &self,
    config: &[KeybindingConfig],
    binding_modes: &[BindingModeConfig],
    paused: bool,
  ) -> anyhow::Result<()> {
    if let Some(event_loop) = &self.event_loop {
      KeyboardHook::update(config, binding_modes, paused, event_loop)?;
    } else {
      tracing::warn!(
        "Event loop is not initialized, cannot update keybinds"
      );
    }

    Ok(())
  }

  pub fn update_mouse(&self, enable_mouse_events: bool) {
    MouseHook::update(enable_mouse_events);
  }
}
