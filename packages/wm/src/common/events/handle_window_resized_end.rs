use tracing::info;
use crate::common::LengthValue;
use crate::windows::commands::resize_window;
use crate::windows::TilingWindow;
use crate::wm_state::WmState;

/// Handles window resize events
pub fn window_resized_end(
    window: TilingWindow,
    state: &mut WmState,
    width_delta: i32,
    height_delta: i32,
) -> anyhow::Result<()> {
    info!("Tiling window resized");
    resize_window(
        window.clone().into(),
        Some(LengthValue::from_px(width_delta)),
        Some(LengthValue::from_px(height_delta)),
        state,
    )
}
