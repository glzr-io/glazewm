//! Windows implementation of [`TabStrip`](crate::TabStrip).
//!
//! Each strip is a `WS_POPUP` window that is excluded from the taskbar
//! and task switcher, and never activates on click. Strips are painted
//! with GDI. Render state is stored per window handle on the event loop
//! thread.

use std::{cell::RefCell, collections::HashMap};

use tokio::sync::mpsc;
use windows::{
  core::w,
  Win32::{
    Foundation::{
      COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
    },
    Graphics::Gdi::{
      BeginPaint, CombineRgn, CreateFontW, CreateRectRgn,
      CreateRoundRectRgn, CreateSolidBrush, DeleteObject, DrawTextW,
      EndPaint, FillRect, InvalidateRect, SelectObject, SetBkMode,
      SetTextColor, SetWindowRgn, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DEFAULT_PITCH, DT_CENTER, DT_SINGLELINE,
      DT_VCENTER, FF_DONTCARE, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS,
      PAINTSTRUCT, RGN_OR, TRANSPARENT,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
      LoadCursorW, RegisterClassW, SetWindowPos, ShowWindow, CS_HREDRAW,
      CS_VREDRAW, HWND_TOP, IDC_ARROW, MA_NOACTIVATE, SWP_NOACTIVATE,
      SWP_NOOWNERZORDER, SW_HIDE, SW_SHOWNOACTIVATE, WM_ERASEBKGND,
      WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_PAINT, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    },
  },
};

use crate::{Dispatcher, TabStripEvent, TabStripId, TabStripState};

thread_local! {
  /// Render state for each strip, keyed by window handle.
  ///
  /// Only accessed on the event loop thread.
  static RENDER_STATES: RefCell<HashMap<isize, RenderState>> =
    RefCell::new(HashMap::new());

  /// Sender for strip events. Only accessed on the event loop thread.
  static EVENT_TX: RefCell<Option<mpsc::UnboundedSender<TabStripEvent>>> =
    const { RefCell::new(None) };
}

/// Platform-specific implementation of [`TabStrip`](crate::TabStrip).
pub(crate) struct TabStrip {
  hwnd: isize,
  dispatcher: Dispatcher,
  last_state: Option<TabStripState>,
}

impl TabStrip {
  /// Implements [`TabStrip::new`](crate::TabStrip::new).
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let hwnd = dispatcher.dispatch_sync(create_window)??;

    Ok(Self {
      hwnd,
      dispatcher: dispatcher.clone(),
      last_state: None,
    })
  }

  /// Implements [`TabStrip::id`](crate::TabStrip::id).
  pub(crate) fn id(&self) -> TabStripId {
    TabStripId(self.hwnd)
  }

  /// Implements [`TabStrip::set_state`](crate::TabStrip::set_state).
  pub(crate) fn set_state(
    &mut self,
    state: &TabStripState,
  ) -> crate::Result<()> {
    if self.last_state.as_ref() == Some(state) {
      return Ok(());
    }

    let hwnd = self.hwnd;
    let owned_state = state.clone();

    self
      .dispatcher
      .dispatch_sync(move || apply_state(hwnd, &owned_state))??;

    self.last_state = Some(state.clone());
    Ok(())
  }
}

impl Drop for TabStrip {
  fn drop(&mut self) {
    let hwnd = self.hwnd;

    if let Err(err) =
      self.dispatcher.dispatch_async(move || destroy_window(hwnd))
    {
      tracing::warn!("Failed to destroy tab strip: {err}");
    }
  }
}

/// Platform-specific implementation of
/// [`TabStripListener`](crate::TabStripListener).
pub(crate) struct TabStripListener;

impl TabStripListener {
  /// Implements [`TabStripListener::new`](crate::TabStripListener::new).
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<TabStripEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    dispatcher.dispatch_sync(move || {
      EVENT_TX.with(|tx| *tx.borrow_mut() = Some(event_tx));
    })?;

    Ok(Self)
  }
}

/// Render state for a strip. Colors are stored as `COLORREF` values.
#[derive(Clone, Debug)]
struct RenderState {
  tab_count: usize,
  active_index: usize,
  active_color: u32,
  inactive_color: u32,
  text_color: u32,
  show_labels: bool,
}

/// Clamps a corner radius so the rounded region fits within the strip.
fn clamp_corner_radius(radius: i32, width: i32, height: i32) -> i32 {
  radius.clamp(0, (width.min(height) / 2).max(0))
}

impl From<&TabStripState> for RenderState {
  fn from(state: &TabStripState) -> Self {
    Self {
      tab_count: state.tab_count,
      active_index: state.active_index,
      active_color: state.active_color.to_bgr(),
      inactive_color: state.inactive_color.to_bgr(),
      text_color: state.text_color.to_bgr(),
      show_labels: state.show_labels,
    }
  }
}

/// Creates a hidden tab strip window.
///
/// Returns the window handle. Must be called on the event loop thread.
fn create_window() -> crate::Result<isize> {
  // Without a class cursor, Windows shows the busy cursor over the strip.
  let arrow_cursor =
    unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default();

  let wnd_class = WNDCLASSW {
    lpszClassName: w!("GlazeWMTabStrip"),
    style: CS_HREDRAW | CS_VREDRAW,
    lpfnWndProc: Some(tab_strip_proc),
    hCursor: arrow_cursor,
    ..Default::default()
  };

  // Registration fails if the class already exists, which is expected
  // after the first strip is created.
  unsafe { RegisterClassW(&raw const wnd_class) };

  let handle = unsafe {
    CreateWindowExW(
      WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
      w!("GlazeWMTabStrip"),
      w!("GlazeWM Tab Strip"),
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      HINSTANCE::default(),
      None,
    )
  };

  if handle.0 == 0 {
    return Err(crate::Error::Platform(
      "Creation of tab strip window failed.".to_string(),
    ));
  }

  Ok(handle.0)
}

/// Applies the given state to a strip window and repaints it.
///
/// Must be called on the event loop thread.
fn apply_state(hwnd: isize, state: &TabStripState) -> crate::Result<()> {
  RENDER_STATES.with(|states| {
    states.borrow_mut().insert(hwnd, RenderState::from(state))
  });

  let rect = &state.rect;
  let insert_after = state
    .z_order_after
    .map_or(HWND_TOP, |window_id| HWND(window_id.0));

  unsafe {
    SetWindowPos(
      HWND(hwnd),
      insert_after,
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
      SWP_NOACTIVATE | SWP_NOOWNERZORDER,
    )
  }?;

  apply_bottom_rounded_region(
    hwnd,
    rect.width(),
    rect.height(),
    state.corner_radius,
  );

  let show_command = if state.is_visible {
    SW_SHOWNOACTIVATE
  } else {
    SW_HIDE
  };

  unsafe { ShowWindow(HWND(hwnd), show_command) };
  unsafe { InvalidateRect(HWND(hwnd), None, true) };

  Ok(())
}

/// Clips a strip window to a region with square top corners and rounded
/// bottom corners.
///
/// Passing a radius of `0` removes the region, which gives square corners.
/// Must be called on the event loop thread.
fn apply_bottom_rounded_region(
  hwnd: isize,
  width: i32,
  height: i32,
  radius: i32,
) {
  let radius = clamp_corner_radius(radius, width, height);

  if radius == 0 {
    // Clears the region. The return value is 0 if there was no region,
    // which is not an error.
    unsafe { SetWindowRgn(HWND(hwnd), None, true) };
    return;
  }

  // The rectangle covers everything above the rounded part, which keeps
  // the top corners square. The rounded rectangle supplies the bottom
  // corners. Their union is the strip's shape.
  let square_part = unsafe { CreateRectRgn(0, 0, width, height - radius) };
  let rounded_part = unsafe {
    CreateRoundRectRgn(0, 0, width, height, radius * 2, radius * 2)
  };
  let combined = unsafe { CreateRectRgn(0, 0, 0, 0) };

  unsafe { CombineRgn(combined, square_part, rounded_part, RGN_OR) };
  unsafe { DeleteObject(HGDIOBJ(square_part.0)) };
  unsafe { DeleteObject(HGDIOBJ(rounded_part.0)) };

  // On success, the system takes ownership of `combined`, so it must not
  // be deleted here.
  if unsafe { SetWindowRgn(HWND(hwnd), combined, true) } == 0 {
    tracing::warn!("Failed to set tab strip window region.");
  }
}

/// Removes a strip's render state and destroys its window.
///
/// Must be called on the event loop thread.
fn destroy_window(hwnd: isize) {
  RENDER_STATES.with(|states| states.borrow_mut().remove(&hwnd));

  if let Err(err) = unsafe { DestroyWindow(HWND(hwnd)) } {
    tracing::warn!("Failed to destroy tab strip window: {err}");
  }
}

/// Result returned for `WM_MOUSEACTIVATE` to prevent activating the strip.
#[allow(clippy::cast_possible_wrap)]
const MA_NOACTIVATE_RESULT: isize = MA_NOACTIVATE as isize;

/// Window procedure for tab strip windows.
unsafe extern "system" fn tab_strip_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    // Painting is fully handled in `WM_PAINT`, so skip background erase
    // to avoid flicker.
    WM_ERASEBKGND => LRESULT(1),
    WM_PAINT => {
      paint(hwnd);
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      handle_click(hwnd, lparam);
      LRESULT(0)
    }
    // Clicks should not steal focus from the active page.
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE_RESULT),
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

/// Paints the tabs of a strip window.
fn paint(hwnd: HWND) {
  let mut paint_struct = PAINTSTRUCT::default();
  let hdc = unsafe { BeginPaint(hwnd, &raw mut paint_struct) };

  let render_state =
    RENDER_STATES.with(|states| states.borrow().get(&hwnd.0).cloned());

  if let (Some(render_state), Some(client)) =
    (render_state, client_rect(hwnd))
  {
    fill_rect(hdc, &client, render_state.inactive_color);

    let width = client.right - client.left;
    let height = client.bottom - client.top;
    let label_font = create_label_font(height);

    for index in 0..render_state.tab_count {
      let (left, right) = tab_bounds(index, render_state.tab_count, width);
      let tab_rect = RECT {
        left,
        top: 0,
        right,
        bottom: height,
      };

      if index == render_state.active_index {
        fill_rect(hdc, &tab_rect, render_state.active_color);
      }

      if render_state.show_labels {
        draw_label(
          hdc,
          &tab_rect,
          index + 1,
          render_state.text_color,
          label_font,
        );
      }
    }

    unsafe { DeleteObject(HGDIOBJ(label_font.0)) };
  }

  unsafe { EndPaint(hwnd, &raw const paint_struct) };
}

/// Fills a rectangle with a solid color.
fn fill_rect(hdc: HDC, rect: &RECT, color: u32) {
  let brush = unsafe { CreateSolidBrush(COLORREF(color)) };

  unsafe { FillRect(hdc, rect, brush) };
  unsafe { DeleteObject(HGDIOBJ(brush.0)) };
}

/// Draws a 1-based tab label centered in the given rectangle.
fn draw_label(
  hdc: HDC,
  rect: &RECT,
  label: usize,
  text_color: u32,
  font: HFONT,
) {
  let mut label_wide =
    label.to_string().encode_utf16().collect::<Vec<u16>>();
  let mut label_rect = *rect;

  unsafe {
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, COLORREF(text_color));

    let previous_font = SelectObject(hdc, HGDIOBJ(font.0));

    DrawTextW(
      hdc,
      &mut label_wide,
      &raw mut label_rect,
      DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    SelectObject(hdc, previous_font);
  }
}

/// Font weight for bold labels (`FW_BOLD`).
const FW_BOLD_WEIGHT: i32 = 700;

/// Creates a bold label font sized relative to the strip height.
///
/// The caller must delete the returned font.
fn create_label_font(strip_height: i32) -> HFONT {
  // A negative height selects the character height rather than the cell
  // height. Scale it to leave some padding within the strip.
  let font_height = -(strip_height * 3 / 5).max(1);

  unsafe {
    CreateFontW(
      font_height,
      0,
      0,
      0,
      FW_BOLD_WEIGHT,
      0,
      0,
      0,
      u32::from(DEFAULT_CHARSET.0),
      u32::from(OUT_DEFAULT_PRECIS.0),
      u32::from(CLIP_DEFAULT_PRECIS.0),
      u32::from(CLEARTYPE_QUALITY.0),
      u32::from(DEFAULT_PITCH.0 | FF_DONTCARE.0),
      w!("Segoe UI"),
    )
  }
}

/// Handles a left click on a strip by emitting a `TabClicked` event.
fn handle_click(hwnd: HWND, lparam: LPARAM) {
  // The x position is stored in the low word as a signed value.
  #[allow(clippy::cast_possible_truncation)]
  let click_x = i32::from(lparam.0 as i16);

  let Some(client) = client_rect(hwnd) else {
    return;
  };

  let tab_count = RENDER_STATES.with(|states| {
    states.borrow().get(&hwnd.0).map(|state| state.tab_count)
  });

  let Some(tab_count) = tab_count else {
    return;
  };

  let width = client.right - client.left;
  let Some(index) = tab_index_at_x(click_x, width, tab_count) else {
    return;
  };

  EVENT_TX.with(|tx| {
    if let Some(tx) = tx.borrow().as_ref() {
      let event = TabStripEvent::TabClicked {
        strip_id: TabStripId(hwnd.0),
        index,
      };

      if tx.send(event).is_err() {
        tracing::warn!("Failed to send tab strip event.");
      }
    }
  });
}

/// Gets the client rectangle of a window.
fn client_rect(hwnd: HWND) -> Option<RECT> {
  let mut client = RECT::default();

  unsafe { GetClientRect(hwnd, &raw mut client) }
    .ok()
    .map(|()| client)
}

/// Gets the horizontal bounds `(left, right)` of a tab in a strip.
///
/// Tabs split the strip's width evenly, with any remainder distributed
/// across tabs.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn tab_bounds(index: usize, tab_count: usize, width: i32) -> (i32, i32) {
  let width = i64::from(width.max(0));
  let count = tab_count.max(1) as i64;

  let left = width * index as i64 / count;
  let right = width * (index as i64 + 1) / count;

  (left as i32, right as i32)
}

/// Gets the index of the tab at the given x position, if any.
fn tab_index_at_x(x: i32, width: i32, tab_count: usize) -> Option<usize> {
  (0..tab_count).find(|&index| {
    let (left, right) = tab_bounds(index, tab_count, width);
    x >= left && x < right
  })
}

#[cfg(test)]
mod tests {
  use super::{tab_bounds, tab_index_at_x, TabStrip};
  use crate::{Color, EventLoop, Rect, TabStripState};

  #[test]
  fn creates_updates_and_destroys_tab_strip() {
    let (event_loop, dispatcher) = EventLoop::new().unwrap();

    let state = TabStripState {
      rect: Rect::from_xy(0, 0, 300, 24),
      tab_count: 2,
      active_index: 1,
      active_color: Color {
        r: 10,
        g: 20,
        b: 30,
        a: 255,
      },
      inactive_color: Color {
        r: 40,
        g: 50,
        b: 60,
        a: 255,
      },
      text_color: Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
      },
      z_order_after: None,
      is_visible: true,
      corner_radius: 8,
      show_labels: true,
    };

    let thread_dispatcher = dispatcher.clone();
    std::thread::spawn(move || {
      let mut tab_strip = TabStrip::new(&thread_dispatcher).unwrap();
      assert_ne!(tab_strip.id().0, 0);

      tab_strip.set_state(&state).unwrap();

      // Setting an unchanged state should be a no-op.
      tab_strip.set_state(&state).unwrap();

      drop(tab_strip);
      thread_dispatcher.stop_event_loop().unwrap();
    });

    event_loop.run().unwrap();
  }

  #[test]
  fn tab_bounds_cover_width_without_gaps() {
    let bounds = (0..3)
      .map(|index| tab_bounds(index, 3, 100))
      .collect::<Vec<_>>();

    assert_eq!(bounds, vec![(0, 33), (33, 66), (66, 100)]);
  }

  #[test]
  fn tab_index_at_x_maps_positions_to_tabs() {
    assert_eq!(tab_index_at_x(0, 100, 2), Some(0));
    assert_eq!(tab_index_at_x(49, 100, 2), Some(0));
    assert_eq!(tab_index_at_x(50, 100, 2), Some(1));
    assert_eq!(tab_index_at_x(99, 100, 2), Some(1));
  }

  #[test]
  fn tab_index_at_x_returns_none_outside_strip() {
    assert_eq!(tab_index_at_x(-1, 100, 2), None);
    assert_eq!(tab_index_at_x(100, 100, 2), None);
    assert_eq!(tab_index_at_x(10, 100, 0), None);
  }
}

#[cfg(test)]
mod corner_tests {
  use super::clamp_corner_radius;

  #[test]
  fn corner_radius_is_clamped_to_fit_strip() {
    assert_eq!(clamp_corner_radius(8, 300, 24), 8);
    assert_eq!(clamp_corner_radius(30, 300, 24), 12);
    assert_eq!(clamp_corner_radius(-4, 300, 24), 0);
  }
}
