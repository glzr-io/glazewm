use anyhow::Result;
use tracing::info;
use wm_common::OpacityValue;

use crate::wm_state::WmState;
use crate::user_config::UserConfig;
use crate::commands::general::platform_sync;
use crate::traits::WindowGetters; 

pub fn toggle_transparency(
    state: &mut WmState,
    config: &mut UserConfig,
) -> Result<()> {
    // Flip the global transparency flag
    state.transparency_enabled = !state.transparency_enabled;
    let enabled = state.transparency_enabled;

    if enabled {
        info!("Transparency ON");
        // Restore transparency settings to their original config values
        // (assuming they were enabled in the original config)
        config.value.window_effects.other_windows.transparency.enabled = true;
    } else {
        info!("Transparency OFF, forcing opaque render");
        // Disable transparency for both focused and unfocused windows
        config.value.window_effects.other_windows.transparency.enabled = false;
        config.value.window_effects.focused_window.transparency.enabled = false;

        // Use fully opaque value (u8::MAX = 255 = 100% opacity)
        let fully_opaque = OpacityValue::from_alpha(u8::MAX);

        // Force all existing windows fully opaque, logging any errors
        for window in state.windows() {
            if let Err(e) = window.native().set_transparency(&fully_opaque) {
                info!("Failed to set window opacity: {:?}", e);
            }
        }
    }

    // Queue redraw and effect updates
    state.pending_sync.queue_containers_to_redraw(state.windows());
    state.pending_sync.queue_all_effects_update();
    state.pending_sync.queue_focused_effect_update();

    // Apply platform-specific sync
    platform_sync(state, config)?;

    Ok(()) 
}
