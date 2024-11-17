use anyhow::Context;
use tracing::warn;
use windows::{
  core::PWSTR,
  Win32::{
    Foundation::{CloseHandle, BOOL, HWND, LPARAM, RECT},
    Graphics::Dwm::{
      DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_BORDER_COLOR,
      DWMWA_CLOAKED, DWMWA_COLOR_NONE, DWMWA_EXTENDED_FRAME_BOUNDS,
      DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DEFAULT, DWMWCP_DONOTROUND,
      DWMWCP_ROUND, DWMWCP_ROUNDSMALL,
    },
    System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
      PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::{
      Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_MOUSE},
      WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindow, GetWindowLongPtrW,
        GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, IsZoomed, SendNotifyMessageW,
        SetForegroundWindow, SetWindowLongPtrW, SetWindowPlacement,
        SetWindowPos, ShowWindowAsync, GWL_EXSTYLE, GWL_STYLE, GW_OWNER,
        HWND_NOTOPMOST, HWND_TOPMOST, SWP_ASYNCWINDOWPOS,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER,
        SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOWNA,
        WINDOWPLACEMENT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE,
        WPF_ASYNCWINDOWPLACEMENT, WS_CAPTION, WS_CHILD, WS_DLGFRAME,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_THICKFRAME,
      },
    },
  },
};

use super::{iapplication_view_collection, iservice_provider, COM_INIT};
use crate::{
  common::{Color, LengthValue, Memo, Rect, RectDelta},
  user_config::{CornerStyle, HideMethod},
  windows::WindowState,
};

#[derive(Debug, Clone)]
pub struct NativeWindow {
  pub handle: isize,
  title: Memo<String>,
  process_name: Memo<String>,
  class_name: Memo<String>,
  frame_position: Memo<Rect>,
  border_position: Memo<Rect>,
  is_minimized: Memo<bool>,
  is_maximized: Memo<bool>,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  pub fn new(handle: isize) -> Self {
    Self {
      handle,
      title: Memo::new(),
      process_name: Memo::new(),
      class_name: Memo::new(),
      frame_position: Memo::new(),
      border_position: Memo::new(),
      is_minimized: Memo::new(),
      is_maximized: Memo::new(),
    }
  }

  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn title(&self) -> anyhow::Result<String> {
    self.title.get_or_init(Self::updated_title, self)
  }

  /// Updates the cached window title.
  pub fn refresh_title(&self) -> anyhow::Result<String> {
    self.title.update(Self::updated_title, self)
  }

  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  fn updated_title(&self) -> anyhow::Result<String> {
    let mut text: [u16; 512] = [0; 512];
    let length = unsafe { GetWindowTextW(HWND(self.handle), &mut text) };
    Ok(String::from_utf16_lossy(&text[..length as usize]))
  }

  /// Gets the process name associated with the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn process_name(&self) -> anyhow::Result<String> {
    self
      .process_name
      .get_or_init(Self::updated_process_name, self)
  }

  /// Gets the process name associated with the window.
  fn updated_process_name(&self) -> anyhow::Result<String> {
    let mut process_id = 0u32;
    unsafe {
      GetWindowThreadProcessId(HWND(self.handle), Some(&mut process_id));
    }

    let process_handle = unsafe {
      OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
    }?;

    let mut buffer = [0u16; 256];
    let mut length = buffer.len() as u32;
    unsafe {
      QueryFullProcessImageNameW(
        process_handle,
        PROCESS_NAME_WIN32,
        PWSTR(buffer.as_mut_ptr()),
        &mut length,
      )?;

      CloseHandle(process_handle)?;
    };

    let exe_path = String::from_utf16_lossy(&buffer[..length as usize]);

    exe_path
      .split('\\')
      .last()
      .map(|file_name| {
        file_name.split('.').next().unwrap_or(file_name).to_string()
      })
      .context("Failed to parse process name.")
  }

  /// Gets the class name of the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn class_name(&self) -> anyhow::Result<String> {
    self.class_name.get_or_init(Self::updated_class_name, self)
  }

  /// Gets the class name of the window.
  fn updated_class_name(&self) -> anyhow::Result<String> {
    let mut buffer = [0u16; 256];
    let result = unsafe { GetClassNameW(HWND(self.handle), &mut buffer) };

    if result == 0 {
      return Err(windows::core::Error::from_win32().into());
    }

    let class_name = String::from_utf16_lossy(&buffer[..result as usize]);
    Ok(class_name)
  }

  /// Whether the window is actually visible.
  pub fn is_visible(&self) -> anyhow::Result<bool> {
    let is_visible =
      unsafe { IsWindowVisible(HWND(self.handle)) }.as_bool();

    Ok(is_visible && !self.is_cloaked()?)
  }

  /// Whether the window is cloaked. For some UWP apps, `WS_VISIBLE` will
  /// be present even if the window isn't actually visible. The
  /// `DWMWA_CLOAKED` attribute is used to check whether these apps are
  /// visible.
  fn is_cloaked(&self) -> anyhow::Result<bool> {
    let mut cloaked = 0u32;

    unsafe {
      DwmGetWindowAttribute(
        HWND(self.handle),
        DWMWA_CLOAKED,
        &mut cloaked as *mut u32 as _,
        std::mem::size_of::<u32>() as u32,
      )
    }?;

    Ok(cloaked != 0)
  }

  pub fn is_manageable(&self) -> anyhow::Result<bool> {
    // Ignore windows that are hidden.
    if !self.is_visible()? {
      return Ok(false);
    }

    let process_name = self.process_name()?;
    let title = self.process_name()?;

    // TODO: Temporary fix for managing Flow Launcher until a force manage
    // command is added.
    if process_name == "Flow.Launcher" && title == "Flow.Launcher" {
      return Ok(true);
    }

    // Ensure window is top-level (i.e. not a child window). Ignore windows
    // that cannot be focused or if they're unavailable in task switcher
    // (alt+tab menu).
    let is_application_window = !self.has_window_style(WS_CHILD)
      && !self.has_window_style_ex(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);

    if !is_application_window {
      return Ok(false);
    }

    // Some applications spawn top-level windows for menus that should be
    // ignored. This includes the autocomplete popup in Notepad++ and title
    // bar menu in Keepass. Although not foolproof, these can typically be
    // identified by having an owner window and no title bar.
    let is_menu_window =
      unsafe { GetWindow(HWND(self.handle), GW_OWNER) }.0 != 0
        && !self.has_window_style(WS_CAPTION);

    Ok(!is_menu_window)
  }

  /// Whether the window is minimized.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn is_minimized(&self) -> anyhow::Result<bool> {
    self
      .is_minimized
      .get_or_init(Self::updated_is_minimized, self)
  }

  /// Updates the cached minimized status.
  pub fn refresh_is_minimized(&self) -> anyhow::Result<bool> {
    self.is_minimized.update(Self::updated_is_minimized, self)
  }

  /// Whether the window is minimized.
  fn updated_is_minimized(&self) -> anyhow::Result<bool> {
    Ok(unsafe { IsIconic(HWND(self.handle)) }.as_bool())
  }

  /// Whether the window is maximized.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn is_maximized(&self) -> anyhow::Result<bool> {
    self
      .is_maximized
      .get_or_init(Self::updated_is_maximized, self)
  }

  /// Updates the cached maximized status.
  pub fn refresh_is_maximized(&self) -> anyhow::Result<bool> {
    self.is_maximized.update(Self::updated_is_maximized, self)
  }

  /// Whether the window is maximized.
  fn updated_is_maximized(&self) -> anyhow::Result<bool> {
    Ok(unsafe { IsZoomed(HWND(self.handle)) }.as_bool())
  }

  /// Whether the window has resize handles.
  pub fn is_resizable(&self) -> bool {
    self.has_window_style(WS_THICKFRAME)
  }

  /// Whether the window is fullscreen.
  ///
  /// Returns `false` if the window is maximized.
  pub fn is_fullscreen(
    &self,
    monitor_rect: &Rect,
  ) -> anyhow::Result<bool> {
    if self.is_maximized()? {
      return Ok(false);
    }

    let position = self.frame_position()?;

    // Allow for 1px of leeway around edges of monitor.
    Ok(
      position.left <= monitor_rect.left + 1
        && position.top <= monitor_rect.top + 1
        && position.right >= monitor_rect.right - 1
        && position.bottom >= monitor_rect.bottom - 1,
    )
  }

  pub fn set_foreground(&self) -> anyhow::Result<()> {
    let input = [INPUT {
      r#type: INPUT_MOUSE,
      ..Default::default()
    }];

    // Bypass restriction for setting the foreground window by sending an
    // input to our own process first.
    unsafe { SendInput(&input, std::mem::size_of::<INPUT>() as i32) };

    // Set as the foreground window.
    unsafe { SetForegroundWindow(HWND(self.handle)) }.ok()?;

    Ok(())
  }

  pub fn set_border_color(
    &self,
    color: Option<&Color>,
  ) -> anyhow::Result<()> {
    let bgr = match color {
      Some(color) => color.to_bgr()?,
      None => DWMWA_COLOR_NONE,
    };

    unsafe {
      DwmSetWindowAttribute(
        HWND(self.handle),
        DWMWA_BORDER_COLOR,
        &bgr as *const _ as _,
        std::mem::size_of::<u32>() as u32,
      )?;
    }

    Ok(())
  }

  pub fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> anyhow::Result<()> {
    let corner_preference = match corner_style {
      CornerStyle::Default => DWMWCP_DEFAULT,
      CornerStyle::Square => DWMWCP_DONOTROUND,
      CornerStyle::Rounded => DWMWCP_ROUND,
      CornerStyle::SmallRounded => DWMWCP_ROUNDSMALL,
    };

    unsafe {
      DwmSetWindowAttribute(
        HWND(self.handle),
        DWMWA_WINDOW_CORNER_PREFERENCE,
        &(corner_preference.0) as *const _ as _,
        std::mem::size_of::<i32>() as u32,
      )?;
    }

    Ok(())
  }

  pub fn set_title_bar_visibility(
    &self,
    visible: bool,
  ) -> anyhow::Result<()> {
    let style = unsafe { GetWindowLongPtrW(HWND(self.handle), GWL_STYLE) };

    let new_style = match visible {
      true => style | (WS_DLGFRAME.0 as isize),
      false => style & !(WS_DLGFRAME.0 as isize),
    };

    if new_style != style {
      unsafe {
        SetWindowLongPtrW(HWND(self.handle), GWL_STYLE, new_style);
        SetWindowPos(
          HWND(self.handle),
          HWND_NOTOPMOST,
          0,
          0,
          0,
          0,
          SWP_FRAMECHANGED
            | SWP_NOMOVE
            | SWP_NOSIZE
            | SWP_NOZORDER
            | SWP_NOOWNERZORDER
            | SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_NOSENDCHANGING
            | SWP_ASYNCWINDOWPOS,
        )?;
      }
    }

    Ok(())
  }

  /// Gets the window's position, including the window's frame. Excludes
  /// the window's shadow borders.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn frame_position(&self) -> anyhow::Result<Rect> {
    self
      .frame_position
      .get_or_init(Self::updated_frame_position, self)
  }

  /// Updates the cached frame position.
  pub fn refresh_frame_position(&self) -> anyhow::Result<Rect> {
    _ = self.refresh_border_position()?;

    self
      .frame_position
      .update(Self::updated_frame_position, self)
  }

  /// Gets the window's position, including the window's frame. Excludes
  /// the window's shadow borders.
  fn updated_frame_position(&self) -> anyhow::Result<Rect> {
    let mut rect = RECT::default();

    let dwm_res = unsafe {
      DwmGetWindowAttribute(
        HWND(self.handle),
        DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut rect as *mut _ as _,
        std::mem::size_of::<RECT>() as u32,
      )
    };

    match dwm_res {
      Ok(_) => Ok(Rect::from_ltrb(
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
      )),
      _ => {
        warn!("Failed to get window's frame position. Falling back to border position.");
        self.border_position()
      }
    }
  }

  /// Gets the window's position, including the window's frame and
  /// shadow borders.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn border_position(&self) -> anyhow::Result<Rect> {
    self
      .border_position
      .get_or_init(Self::updated_border_position, self)
  }

  /// Updates the cached border position.
  pub fn refresh_border_position(&self) -> anyhow::Result<Rect> {
    self
      .border_position
      .update(Self::updated_border_position, self)
  }

  /// Gets the window's position, including the window's frame and
  /// shadow borders.
  fn updated_border_position(&self) -> anyhow::Result<Rect> {
    let mut rect = RECT::default();

    unsafe { GetWindowRect(HWND(self.handle), &mut rect as *mut _ as _) }?;

    Ok(Rect::from_ltrb(
      rect.left,
      rect.top,
      rect.right,
      rect.bottom,
    ))
  }

  /// Gets the delta between the window's frame and the window's border.
  /// This represents the size of a window's shadow borders.
  pub fn shadow_border_delta(&self) -> anyhow::Result<RectDelta> {
    let border_pos = self.border_position()?;
    let frame_pos = self.frame_position()?;

    Ok(RectDelta::new(
      LengthValue::from_px(frame_pos.left - border_pos.left),
      LengthValue::from_px(frame_pos.top - border_pos.top),
      LengthValue::from_px(border_pos.right - frame_pos.right),
      LengthValue::from_px(border_pos.bottom - frame_pos.bottom),
    ))
  }

  fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(HWND(self.handle), GWL_STYLE) };

    (current_style & style.0 as isize) != 0
  }

  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(HWND(self.handle), GWL_EXSTYLE) };

    (current_style & style.0 as isize) != 0
  }

  pub fn restore_to_position(&self, rect: &Rect) -> anyhow::Result<()> {
    let placement = WINDOWPLACEMENT {
      length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
      flags: WPF_ASYNCWINDOWPLACEMENT,
      showCmd: SW_RESTORE.0 as u32,
      rcNormalPosition: RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
      },
      ..Default::default()
    };

    unsafe { SetWindowPlacement(HWND(self.handle), &placement) }?;
    Ok(())
  }

  pub fn maximize(&self) -> anyhow::Result<()> {
    unsafe { ShowWindowAsync(HWND(self.handle), SW_MAXIMIZE).ok() }?;
    Ok(())
  }

  pub fn minimize(&self) -> anyhow::Result<()> {
    unsafe { ShowWindowAsync(HWND(self.handle), SW_MINIMIZE).ok() }?;
    Ok(())
  }

  pub fn close(&self) -> anyhow::Result<()> {
    unsafe {
      SendNotifyMessageW(HWND(self.handle), WM_CLOSE, None, None)
    }?;

    Ok(())
  }

  pub fn set_visible(
    &self,
    visible: bool,
    hide_method: &HideMethod,
  ) -> anyhow::Result<()> {
    match hide_method {
      HideMethod::Hide => {
        if visible {
          self.show()
        } else {
          self.hide()
        }
      }
      HideMethod::Cloak => self.set_cloaked(!visible),
    }
  }

  pub fn show(&self) -> anyhow::Result<()> {
    unsafe { ShowWindowAsync(HWND(self.handle), SW_SHOWNA) }.ok()?;
    Ok(())
  }

  pub fn hide(&self) -> anyhow::Result<()> {
    unsafe { ShowWindowAsync(HWND(self.handle), SW_HIDE) }.ok()?;
    Ok(())
  }

  pub fn set_cloaked(&self, cloaked: bool) -> anyhow::Result<()> {
    COM_INIT.with(|_| -> anyhow::Result<()> {
      let view_collection =
        iapplication_view_collection(&iservice_provider()?)?;

      let mut view = None;
      unsafe { view_collection.get_view_for_hwnd(self.handle, &mut view) }
        .ok()?;

      let view = view
        .context("Unable to get application view by window handle.")?;

      // Ref: https://github.com/Ciantic/AltTabAccessor/issues/1#issuecomment-1426877843
      unsafe { view.set_cloak(1, if cloaked { 2 } else { 0 }) }
        .ok()
        .context("Failed to cloak window.")
    })
  }

  pub fn set_position(
    &self,
    state: &WindowState,
    rect: &Rect,
    is_visible: bool,
    hide_method: &HideMethod,
    has_pending_dpi_adjustment: bool,
  ) -> anyhow::Result<()> {
    // Restore window if it's minimized/maximized and shouldn't be. This is
    // needed to be able to move and resize it.
    match state {
      // Need to restore window if transitioning from maximized fullscreen
      // to non-maximized fullscreen.
      WindowState::Fullscreen(config) => {
        if !config.maximized && self.is_maximized()? {
          // Restoring to position has the same effect as `ShowWindow` with
          // `SW_RESTORE`, but doesn't cause a flicker.
          self.restore_to_position(rect)?;
        }
      }
      // No need to restore window if it'll be minimized. Transitioning
      // from maximized to minimized works without having to restore.
      WindowState::Minimized => {}
      _ => {
        if self.is_minimized()? || self.is_maximized()? {
          self.restore_to_position(rect)?;
        }
      }
    }

    let mut swp_flags = SWP_NOACTIVATE
      | SWP_NOCOPYBITS
      | SWP_NOSENDCHANGING
      | SWP_ASYNCWINDOWPOS;

    // Whether the window should be shown above all other windows.
    let z_order = match state {
      WindowState::Floating(config) if config.shown_on_top => HWND_TOPMOST,
      WindowState::Fullscreen(config) if config.shown_on_top => {
        HWND_TOPMOST
      }
      _ => HWND_NOTOPMOST,
    };

    match state {
      WindowState::Minimized => {
        if !self.is_minimized()? {
          self.minimize()?;
        }
      }
      WindowState::Fullscreen(config)
        if config.maximized && self.has_window_style(WS_MAXIMIZEBOX) =>
      {
        if !self.is_maximized()? {
          self.maximize()?;
        }

        unsafe {
          SetWindowPos(
            HWND(self.handle),
            z_order,
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
            swp_flags,
          )
        }?;
      }
      _ => {
        swp_flags |= SWP_FRAMECHANGED;

        unsafe {
          SetWindowPos(
            HWND(self.handle),
            z_order,
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
            swp_flags,
          )
        }?;

        // When there's a mismatch between the DPI of the monitor and the
        // window, the window might be sized incorrectly after the first
        // move. If we set the position twice, inconsistencies after the
        // first move are resolved.
        if has_pending_dpi_adjustment {
          unsafe {
            SetWindowPos(
              HWND(self.handle),
              z_order,
              rect.x(),
              rect.y(),
              rect.width(),
              rect.height(),
              swp_flags,
            )
          }?;
        }
      }
    };

    // Whether to show or hide the window.
    self.set_visible(is_visible, hide_method)?;

    Ok(())
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeWindow {}

pub fn available_windows() -> anyhow::Result<Vec<NativeWindow>> {
  available_window_handles()?
    .into_iter()
    .map(|handle| Ok(NativeWindow::new(handle)))
    .collect()
}

pub fn available_window_handles() -> anyhow::Result<Vec<isize>> {
  let mut handles: Vec<isize> = Vec::new();

  unsafe {
    EnumWindows(
      Some(available_window_handles_proc),
      LPARAM(&mut handles as *mut _ as _),
    )
  }?;

  Ok(handles)
}

extern "system" fn available_window_handles_proc(
  handle: HWND,
  data: LPARAM,
) -> BOOL {
  let handles = data.0 as *mut Vec<isize>;
  unsafe { (*handles).push(handle.0) };
  true.into()
}
