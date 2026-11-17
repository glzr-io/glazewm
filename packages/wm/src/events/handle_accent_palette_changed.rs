use tracing::info;
use wm_common::WmEvent;
use wm_platform::AccentPalette;

use crate::{user_config::UserConfig, wm_state::WmState};

/// Handles a change to the system accent palette.
///
/// Re-applies window effects if any border uses an accent shade, and
/// emits the new palette to IPC subscribers.
#[allow(clippy::unnecessary_wraps)]
pub fn handle_accent_palette_changed(
  accent_palette: AccentPalette,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Accent palette changed: {:?}", accent_palette.accent);

  state.accent_palette = Some(accent_palette.clone());

  if config.value.window_effects.has_accent_border() {
    state.pending_sync.queue_all_effects_update();
  }

  state.emit_event(WmEvent::AccentPaletteChanged { accent_palette });

  Ok(())
}
