use std::sync::OnceLock;

use windows::{
  core::{w, ComInterface},
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::{
      Direct2D::{
        Common::{
          D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
          D2D_RECT_F, D2D_SIZE_U,
        },
        D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget,
        ID2D1RenderTarget, ID2D1SolidColorBrush,
        D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        D2D1_FACTORY_TYPE_SINGLE_THREADED,
        D2D1_HWND_RENDER_TARGET_PROPERTIES,
        D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_RENDER_TARGET_PROPERTIES,
        D2D1_RENDER_TARGET_TYPE_DEFAULT,
      },
      Dwm::{
        DwmEnableBlurBehindWindow, DWM_BB_BLURREGION, DWM_BB_ENABLE,
        DWM_BLURBEHIND,
      },
      Dxgi::Common::DXGI_FORMAT_UNKNOWN,
      Gdi::{
        BeginPaint, CombineRgn, CreateRectRgn, DeleteObject, EndPaint,
        InvalidateRect, PAINTSTRUCT, RGN_DIFF, RGN_ERROR, SetWindowRgn,
      },
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
      GetSystemMetrics, GetWindowLongPtrW, LoadCursorW, RegisterClassW,
      SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW,
      GWLP_USERDATA, HTTRANSPARENT, IDC_ARROW, SM_CXVIRTUALSCREEN,
      SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
      SWP_SHOWWINDOW, SW_HIDE, WM_CREATE, WM_DESTROY, WM_ERASEBKGND,
      WM_NCCREATE, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    },
  },
};

use crate::{Color, Dispatcher, Rect, WindowId};

const BORDER_WIDTH: i32 = 4;
const BORDER_OFFSET: i32 = 2;
const OVERLAY_CLASS_NAME: windows::core::PCWSTR =
  w!("GlazeWMFocusedWindowBorder");

/// A Windows-specific focused-window border overlay.
pub(crate) struct FocusedWindowBorder {
  dispatcher: Dispatcher,
  hwnd: isize,
  tracked_window_id: Option<WindowId>,
}

impl FocusedWindowBorder {
  /// Creates a focused-window border overlay.
  pub(crate) fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    let hwnd = dispatcher.dispatch_sync(create_overlay_window)??;

    Ok(Self {
      dispatcher,
      hwnd,
      tracked_window_id: None,
    })
  }

  /// Shows the border around the given window.
  pub(crate) fn show(
    &mut self,
    tracked_window_id: WindowId,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<()> {
    self.tracked_window_id = Some(tracked_window_id);
    let hwnd = self.hwnd;
    let frame = frame.clone();
    let color = color.clone();
    let tracked_hwnd = tracked_window_id.0;

    self.dispatcher.dispatch_sync(move || {
      with_overlay_state(hwnd, |state| {
        state.tracked_hwnd = tracked_hwnd;
        state.frame = frame;
        state.color = color;
        state.show(hwnd)
      })
    })??;

    Ok(())
  }

  /// Updates the border position for the currently tracked window.
  pub(crate) fn update_position(
    &mut self,
    tracked_window_id: WindowId,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<()> {
    if self.tracked_window_id != Some(tracked_window_id) {
      return self.show(tracked_window_id, frame, color);
    }

    let hwnd = self.hwnd;
    let frame = frame.clone();
    let color = color.clone();

    self.dispatcher.dispatch_sync(move || {
      with_overlay_state(hwnd, |state| {
        state.frame = frame;
        state.color = color;
        state.update_position(hwnd)
      })
    })??;

    Ok(())
  }

  /// Hides the border overlay.
  pub(crate) fn hide(&mut self) -> crate::Result<()> {
    self.tracked_window_id = None;
    let hwnd = self.hwnd;

    self.dispatcher.dispatch_sync(move || {
      with_overlay_state(hwnd, |state| state.hide(hwnd))
    })??;

    Ok(())
  }

  /// Destroys the overlay window.
  pub(crate) fn shutdown(&mut self) -> crate::Result<()> {
    self.tracked_window_id = None;

    if self.hwnd == 0 {
      return Ok(());
    }

    let hwnd = self.hwnd;
    self.hwnd = 0;

    self.dispatcher.dispatch_sync(move || unsafe {
      DestroyWindow(HWND(hwnd))?;
      Ok::<(), crate::Error>(())
    })??;

    Ok(())
  }

  /// Gets the currently tracked window ID.
  #[must_use]
  pub(crate) fn tracked_window_id(&self) -> Option<WindowId> {
    self.tracked_window_id
  }
}

struct OverlayState {
  factory: Option<ID2D1Factory>,
  render_target: Option<ID2D1HwndRenderTarget>,
  brush: Option<ID2D1SolidColorBrush>,
  color: Color,
  frame: Rect,
  tracked_hwnd: isize,
}

impl Default for OverlayState {
  fn default() -> Self {
    Self {
      factory: None,
      render_target: None,
      brush: None,
      color: Color {
        r: 140,
        g: 190,
        b: 255,
        a: 255,
      },
      frame: Rect::from_xy(0, 0, 0, 0),
      tracked_hwnd: 0,
    }
  }
}

impl OverlayState {
  /// Shows the overlay window.
  fn show(&mut self, hwnd: isize) -> crate::Result<()> {
    self.position_window(hwnd, true)?;
    self.invalidate(hwnd);
    Ok(())
  }

  /// Updates the overlay position.
  fn update_position(&mut self, hwnd: isize) -> crate::Result<()> {
    self.position_window(hwnd, true)?;
    self.invalidate(hwnd);
    Ok(())
  }

  /// Hides the overlay window.
  fn hide(&mut self, hwnd: isize) -> crate::Result<()> {
    unsafe {
      SetWindowPos(
        HWND(hwnd),
        HWND(0),
        0,
        0,
        0,
        0,
        SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
      )?;
    }

    Ok(())
  }

  /// Positions the overlay window around the tracked frame.
  fn position_window(
    &self,
    hwnd: isize,
    should_show: bool,
  ) -> crate::Result<()> {
    let overlay_rect = overlay_rect(&self.frame);
    let flags = if should_show {
      SWP_NOACTIVATE | SWP_SHOWWINDOW
    } else {
      SWP_NOACTIVATE
    };

    unsafe {
      SetWindowPos(
        HWND(hwnd),
        HWND(-1),
        overlay_rect.left,
        overlay_rect.top,
        overlay_rect.width(),
        overlay_rect.height(),
        flags,
      )?;
    }

    update_window_region(HWND(hwnd), &overlay_rect)?;

    Ok(())
  }

  /// Invalidates the overlay window to trigger repaint.
  fn invalidate(&self, hwnd: isize) {
    unsafe {
      let _ = InvalidateRect(HWND(hwnd), None, false);
    }
  }

  /// Renders the overlay border.
  fn render(&mut self, hwnd: HWND) -> crate::Result<()> {
    let client_rect = client_rect(hwnd)?;

    if client_rect.right <= 0 || client_rect.bottom <= 0 {
      return Ok(());
    }

    self.ensure_render_target(hwnd, &client_rect)?;
    let render_target = self.render_target.clone().ok_or_else(|| {
      crate::Error::Platform(
        "Direct2D render target was not created.".to_string(),
      )
    })?;
    self.ensure_brush(&render_target)?;
    let brush = self.brush.clone().ok_or_else(|| {
      crate::Error::Platform("Direct2D brush was not created.".to_string())
    })?;

    unsafe {
      render_target.Resize(&D2D_SIZE_U {
        width: client_rect.right as u32,
        height: client_rect.bottom as u32,
      })?;
      render_target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
      render_target.BeginDraw();
      render_target.Clear(Some(&D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
      }));
      render_target.DrawRectangle(
        &D2D_RECT_F {
          left: (BORDER_OFFSET + (BORDER_WIDTH / 2)) as f32,
          top: (BORDER_OFFSET + (BORDER_WIDTH / 2)) as f32,
          right: client_rect.right as f32
            - (BORDER_OFFSET + (BORDER_WIDTH / 2)) as f32,
          bottom: client_rect.bottom as f32
            - (BORDER_OFFSET + (BORDER_WIDTH / 2)) as f32,
        },
        &brush,
        BORDER_WIDTH as f32,
        None,
      );
      let _ = render_target.EndDraw(None, None);
    }

    Ok(())
  }

  /// Gets or creates the Direct2D factory.
  fn ensure_factory(&mut self) -> crate::Result<&ID2D1Factory> {
    if self.factory.is_none() {
      let factory = unsafe {
        D2D1CreateFactory::<ID2D1Factory>(
          D2D1_FACTORY_TYPE_SINGLE_THREADED,
          None,
        )
      }?;

      self.factory = Some(factory);
    }

    self.factory.as_ref().ok_or_else(|| {
      crate::Error::Platform(
        "Direct2D factory was not created.".to_string(),
      )
    })
  }

  /// Gets or creates the Direct2D render target for the overlay window.
  fn ensure_render_target(
    &mut self,
    hwnd: HWND,
    client_rect: &RECT,
  ) -> crate::Result<&ID2D1HwndRenderTarget> {
    if self.render_target.is_none() {
      let factory = self.ensure_factory()?;
      let render_target_properties = D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: D2D1_PIXEL_FORMAT {
          format: DXGI_FORMAT_UNKNOWN,
          alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        ..Default::default()
      };
      let hwnd_render_target_properties =
        D2D1_HWND_RENDER_TARGET_PROPERTIES {
          hwnd,
          pixelSize: D2D_SIZE_U {
            width: client_rect.right as u32,
            height: client_rect.bottom as u32,
          },
          presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
        };

      let render_target = unsafe {
        factory.CreateHwndRenderTarget(
          &render_target_properties,
          &hwnd_render_target_properties,
        )
      }?;

      self.render_target = Some(render_target);
    }

    self.render_target.as_ref().ok_or_else(|| {
      crate::Error::Platform(
        "Direct2D render target was not created.".to_string(),
      )
    })
  }

  /// Gets or creates the brush for the current border color.
  fn ensure_brush(
    &mut self,
    render_target: &ID2D1HwndRenderTarget,
  ) -> crate::Result<&ID2D1SolidColorBrush> {
    if self.brush.is_none() {
      let render_target: ID2D1RenderTarget = render_target.cast()?;
      let color = d2d_color(&self.color);
      let brush =
        unsafe { render_target.CreateSolidColorBrush(&color, None) }?;
      self.brush = Some(brush);
    } else if let Some(brush) = &self.brush {
      unsafe {
        brush.SetColor(&d2d_color(&self.color));
      }
    }

    self.brush.as_ref().ok_or_else(|| {
      crate::Error::Platform("Direct2D brush was not created.".to_string())
    })
  }
}

/// Creates the hidden overlay window on the event-loop thread.
fn create_overlay_window() -> crate::Result<isize> {
  register_overlay_class()?;

  let overlay_state = Box::new(OverlayState::default());
  let overlay_state_ptr = Box::into_raw(overlay_state);

  let hwnd = unsafe {
    // `ID2D1HwndRenderTarget` does not render reliably on Win10 layered
    // popup windows, so transparency is provided via DWM blur-behind
    // instead of `WS_EX_LAYERED`.
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
      OVERLAY_CLASS_NAME,
      OVERLAY_CLASS_NAME,
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      None,
      Some(overlay_state_ptr.cast()),
    )
  };

  if hwnd.0 == 0 {
    unsafe {
      drop(Box::from_raw(overlay_state_ptr));
    }

    return Err(crate::Error::Platform(
      "Failed to create focused-window border overlay.".to_string(),
    ));
  }

  enable_transparent_background(hwnd)?;

  unsafe {
    ShowWindow(hwnd, SW_HIDE);
  }

  Ok(hwnd.0)
}

/// Registers the overlay window class once.
fn register_overlay_class() -> crate::Result<()> {
  static REGISTERED: OnceLock<()> = OnceLock::new();

  if REGISTERED.get().is_some() {
    return Ok(());
  }

  let window_class = WNDCLASSW {
    lpszClassName: OVERLAY_CLASS_NAME,
    lpfnWndProc: Some(overlay_window_proc),
    hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
    ..Default::default()
  };

  unsafe {
    RegisterClassW(&window_class);
  }

  let _ = REGISTERED.set(());

  Ok(())
}

/// Window procedure for the focused-window border overlay.
unsafe extern "system" fn overlay_window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_NCCREATE => {
      let create_struct = lparam.0 as *const CREATESTRUCTW;
      let state_ptr = (*create_struct).lpCreateParams as *mut OverlayState;
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
      LRESULT(1)
    }
    WM_CREATE => LRESULT(0),
    WM_ERASEBKGND => LRESULT(1),
    WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
    WM_PAINT => {
      let mut paint = PAINTSTRUCT::default();
      BeginPaint(hwnd, &mut paint);

      if let Err(err) =
        with_overlay_state(hwnd.0, |state| state.render(hwnd))
      {
        tracing::warn!("Failed to render focused-window border: {err}");
      }

      EndPaint(hwnd, &paint);
      LRESULT(0)
    }
    WM_DESTROY => LRESULT(0),
    WM_NCDESTROY => {
      let state_ptr = overlay_state_ptr(hwnd);

      if !state_ptr.is_null() {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        drop(Box::from_raw(state_ptr));
      }

      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

/// Enables transparent background rendering for the overlay window.
fn enable_transparent_background(hwnd: HWND) -> crate::Result<()> {
  unsafe {
    let pos = -GetSystemMetrics(SM_CXVIRTUALSCREEN) - 8;
    let region = CreateRectRgn(pos, 0, pos + 1, 1);
    let blur = DWM_BLURBEHIND {
      dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION,
      fEnable: true.into(),
      hRgnBlur: region,
      fTransitionOnMaximized: false.into(),
    };

    DwmEnableBlurBehindWindow(hwnd, &blur)?;
  }

  Ok(())
}

/// Calls the given closure with mutable access to the overlay state.
fn with_overlay_state<R>(
  hwnd: isize,
  callback: impl FnOnce(&mut OverlayState) -> crate::Result<R>,
) -> crate::Result<R> {
  let state_ptr = overlay_state_ptr(HWND(hwnd));

  if state_ptr.is_null() {
    return Err(crate::Error::WindowNotFound);
  }

  unsafe { callback(&mut *state_ptr) }
}

/// Gets the overlay state pointer stored on the window.
fn overlay_state_ptr(hwnd: HWND) -> *mut OverlayState {
  unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState }
}

/// Converts a window frame to the overlay window rect.
#[must_use]
fn overlay_rect(frame: &Rect) -> Rect {
  frame.inset(-(BORDER_OFFSET + BORDER_WIDTH))
}

/// Gets the current client rect for the given window.
fn client_rect(hwnd: HWND) -> crate::Result<RECT> {
  let mut rect = RECT::default();

  unsafe {
    GetClientRect(hwnd, &mut rect)?;
  }

  Ok(rect)
}

/// Updates the overlay window region so only the border ring is hit-testable.
fn update_window_region(hwnd: HWND, overlay_rect: &Rect) -> crate::Result<()> {
  let outer_width = overlay_rect.width().max(0);
  let outer_height = overlay_rect.height().max(0);
  let inset = BORDER_OFFSET + BORDER_WIDTH;
  let inner_left = inset.min(outer_width);
  let inner_top = inset.min(outer_height);
  let inner_right = (outer_width - inset).max(inner_left);
  let inner_bottom = (outer_height - inset).max(inner_top);

  // SAFETY: Region handles are created for the current thread and are either
  // released with `DeleteObject` on failure paths or transferred to Windows
  // with `SetWindowRgn` on success.
  let outer_region = unsafe {
    CreateRectRgn(0, 0, outer_width, outer_height)
  };
  // SAFETY: The inner rectangle is clamped to the outer rectangle bounds,
  // so the region coordinates remain valid even for very small windows.
  let inner_region = unsafe {
    CreateRectRgn(
      inner_left,
      inner_top,
      inner_right,
      inner_bottom,
    )
  };

  // SAFETY: The region handles are valid when created above. Ownership of
  // `outer_region` transfers to the OS on `SetWindowRgn` success.
  unsafe {
    let combine_result =
      CombineRgn(outer_region, outer_region, inner_region, RGN_DIFF);
    let _ = DeleteObject(inner_region);

    if combine_result == RGN_ERROR {
      let _ = DeleteObject(outer_region);
      return Err(crate::Error::Platform(
        "Failed to update focused-window border region.".to_string(),
      ));
    }

    if SetWindowRgn(hwnd, outer_region, true) == 0 {
      let _ = DeleteObject(outer_region);
      return Err(crate::Error::Platform(
        "Failed to apply focused-window border region.".to_string(),
      ));
    }
  }

  Ok(())
}

/// Converts a `Color` to a Direct2D color.
#[must_use]
fn d2d_color(color: &Color) -> D2D1_COLOR_F {
  D2D1_COLOR_F {
    r: f32::from(color.r) / 255.0,
    g: f32::from(color.g) / 255.0,
    b: f32::from(color.b) / 255.0,
    a: f32::from(color.a) / 255.0,
  }
}
