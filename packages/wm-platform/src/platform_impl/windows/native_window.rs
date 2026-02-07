use std::time::Duration;

use tokio::task;
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
      Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT,
      },
      WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetLayeredWindowAttributes, GetWindow,
        GetWindowLongPtrW, GetWindowRect, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
        SendNotifyMessageW, SetForegroundWindow,
        SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPlacement,
        SetWindowPos, ShowWindowAsync, GWL_EXSTYLE, GWL_STYLE, GW_OWNER,
        HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST,
        LAYERED_WINDOW_ATTRIBUTES_FLAGS, LWA_ALPHA, LWA_COLORKEY,
        SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE,
        SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSENDCHANGING,
        SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_MAXIMIZE,
        SW_MINIMIZE, SW_RESTORE, SW_SHOWNA, WINDOWPLACEMENT,
        WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WPF_ASYNCWINDOWPLACEMENT,
        WS_CAPTION, WS_CHILD, WS_DLGFRAME, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_THICKFRAME,
      },
    },
  },
};

use super::COM_INIT;
use crate::{
  Color, CornerStyle, Delta, HideMethod, LengthValue, OpacityValue, Rect,
  RectDelta, WindowState, ZOrder,
};

/// Magic number used to identify programmatic mouse inputs from our own
/// process.
pub(crate) const FOREGROUND_INPUT_IDENTIFIER: u32 = 6379;

pub trait NativeWindowWindowsExt {
  fn class_name(&self) -> crate::Result<String>;
  fn process_name(&self) -> crate::Result<String>;
  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;
  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()>;
  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()>;
  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()>;
  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()>;

  /// Gets the window handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hwnd(&self) -> HWND;

  /// Sets the window's z-order.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_z_order(&self, zorder: &ZOrder) -> crate::Result<()>;

  /// Restores the window (unminimizes and unmaximizes).
  ///
  /// If `outer_frame` is provided, the window will be restored to the
  /// specified position. This avoids flickering compared to restoring
  /// and then repositioning the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()>;

  /// Gets the window's frame, including the window's shadow borders.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  ///
  /// TODO: Consider renaming this to `outer_frame`.`
  fn frame_with_shadows(&self) -> crate::Result<Rect>;
}

impl NativeWindowWindowsExt for NativeWindow {
  fn hwnd(&self) -> HWND {
    HWND(self.handle)
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  handle: isize,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub(crate) fn new(handle: isize) -> Self {
    Self { handle }
  }

  /// Gets the window's title. If the window is invalid, returns an
  /// empty string.
  pub(crate) fn title(&self) -> crate::Result<String> {
    if !unsafe { IsWindow(self.hwnd()) }.as_bool() {
      return crate::Error::WindowNotFound;
    }

    let mut text: [u16; 512] = [0; 512];
    let length = unsafe { GetWindowTextW(self.hwnd(), &mut text) };

    #[allow(clippy::cast_sign_loss)]
    Ok(String::from_utf16_lossy(&text[..length as usize]))
  }

  /// Gets the process name associated with the window.
  pub(crate) fn process_name(&self) -> crate::Result<String> {
    let mut process_id = 0u32;
    unsafe {
      GetWindowThreadProcessId(self.hwnd(), Some(&raw mut process_id));
    }

    let process_handle = unsafe {
      OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
    }?;

    let mut buffer = [0u16; 256];
    let mut length = u32::try_from(buffer.len())?;
    unsafe {
      QueryFullProcessImageNameW(
        process_handle,
        PROCESS_NAME_WIN32,
        PWSTR(buffer.as_mut_ptr()),
        &raw mut length,
      )?;

      CloseHandle(process_handle)?;
    };

    let exe_path = String::from_utf16_lossy(&buffer[..length as usize]);

    exe_path
      .split('\\')
      .next_back()
      .map(|file_name| {
        file_name.split('.').next().unwrap_or(file_name).to_string()
      })
      .ok_or_else(|| {
        crate::Error::Platform("Failed to parse process name.".to_string())
      })
  }

  /// Gets the class name of the window.
  pub(crate) fn class_name(&self) -> crate::Result<String> {
    let mut buffer = [0u16; 256];
    let result = unsafe { GetClassNameW(self.hwnd(), &mut buffer) };

    if result == 0 {
      return Err(windows::core::Error::from_win32().into());
    }

    #[allow(clippy::cast_sign_loss)]
    let class_name = String::from_utf16_lossy(&buffer[..result as usize]);
    Ok(class_name)
  }

  /// Whether the window is actually visible.
  pub(crate) fn is_visible(&self) -> crate::Result<bool> {
    let is_visible = unsafe { IsWindowVisible(self.hwnd()) }.as_bool();

    Ok(is_visible && !self.is_cloaked()?)
  }

  /// Whether the window is cloaked. For some UWP apps, `WS_VISIBLE` will
  /// be present even if the window isn't actually visible. The
  /// `DWMWA_CLOAKED` attribute is used to check whether these apps are
  /// visible.
  fn is_cloaked(&self) -> crate::Result<bool> {
    let mut cloaked = 0u32;

    unsafe {
      #[allow(clippy::cast_possible_truncation)]
      DwmGetWindowAttribute(
        self.hwnd(),
        DWMWA_CLOAKED,
        std::ptr::from_mut::<u32>(&mut cloaked).cast(),
        std::mem::size_of::<u32>() as u32,
      )
    }?;

    Ok(cloaked != 0)
  }

  // TODO: Should probably be removed and have its logic called explicitly.
  pub(crate) fn is_manageable(&self) -> crate::Result<bool> {
    // Ignore windows that are hidden.
    if !self.is_visible()? {
      return Ok(false);
    }

    // Ensure window has a valid process name, title, and class name.
    let process_name = self.process_name()?;
    let title = self.title()?;
    let _ = self.class_name()?;

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

    // Ensure window position is accessible.
    self.invalidate_frame_position()?;

    // Some applications spawn top-level windows for menus that should be
    // ignored. This includes the autocomplete popup in Notepad++ and title
    // bar menu in Keepass. Although not foolproof, these can typically be
    // identified by having an owner window and no title bar.
    let is_menu_window = unsafe { GetWindow(self.hwnd(), GW_OWNER) }.0
      != 0
      && !self.has_window_style(WS_CAPTION);

    Ok(!is_menu_window)
  }

  /// Whether the window is minimized.
  pub(crate) fn is_minimized(&self) -> crate::Result<bool> {
    Ok(unsafe { IsIconic(self.hwnd()) }.as_bool())
  }

  /// Whether the window is maximized.
  pub(crate) fn is_maximized(&self) -> crate::Result<bool> {
    Ok(unsafe { IsZoomed(self.hwnd()) }.as_bool())
  }

  /// Whether the window has resize handles.
  #[must_use]
  pub(crate) fn is_resizable(&self) -> bool {
    self.has_window_style(WS_THICKFRAME)
  }

  /// Windows-specific implementation of [`NativeWindow::focus`].
  pub(crate) fn focus(&self) -> crate::Result<()> {
    let input = [INPUT {
      r#type: INPUT_MOUSE,
      Anonymous: INPUT_0 {
        mi: MOUSEINPUT {
          dwExtraInfo: FOREGROUND_INPUT_IDENTIFIER as usize,
          ..Default::default()
        },
      },
    }];

    // Bypass restriction for setting the foreground window by sending an
    // input to our own process first.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    unsafe {
      SendInput(&input, std::mem::size_of::<INPUT>() as i32)
    };

    // Set as the foreground window.
    unsafe { SetForegroundWindow(self.hwnd()) }.ok()?;

    Ok(())
  }

  pub(crate) fn set_border_color(
    &self,
    color: Option<&Color>,
  ) -> crate::Result<()> {
    let bgr = match color {
      Some(color) => color.to_bgr()?,
      None => DWMWA_COLOR_NONE,
    };

    unsafe {
      #[allow(clippy::cast_possible_truncation)]
      DwmSetWindowAttribute(
        self.hwnd(),
        DWMWA_BORDER_COLOR,
        std::ptr::from_ref(&bgr).cast(),
        std::mem::size_of::<u32>() as u32,
      )?;
    }

    Ok(())
  }

  pub(crate) fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()> {
    let corner_preference = match corner_style {
      CornerStyle::Default => DWMWCP_DEFAULT,
      CornerStyle::Square => DWMWCP_DONOTROUND,
      CornerStyle::Rounded => DWMWCP_ROUND,
      CornerStyle::SmallRounded => DWMWCP_ROUNDSMALL,
    };

    unsafe {
      #[allow(clippy::cast_possible_truncation)]
      DwmSetWindowAttribute(
        self.hwnd(),
        DWMWA_WINDOW_CORNER_PREFERENCE,
        std::ptr::from_ref(&(corner_preference.0)).cast(),
        std::mem::size_of::<i32>() as u32,
      )?;
    }

    Ok(())
  }

  pub(crate) fn set_title_bar_visibility(
    &self,
    visible: bool,
  ) -> crate::Result<()> {
    let style = unsafe { GetWindowLongPtrW(self.hwnd(), GWL_STYLE) };

    #[allow(clippy::cast_possible_wrap)]
    let new_style = if visible {
      style | (WS_DLGFRAME.0 as isize)
    } else {
      style & !(WS_DLGFRAME.0 as isize)
    };

    if new_style != style {
      unsafe {
        SetWindowLongPtrW(self.hwnd(), GWL_STYLE, new_style);
        SetWindowPos(
          self.hwnd(),
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

  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) };

    #[allow(clippy::cast_possible_wrap)]
    if current_style & style.0 as isize == 0 {
      let new_style = current_style | style.0 as isize;

      unsafe { SetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE, new_style) };
    }
  }

  pub(crate) fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()> {
    let mut alpha = u8::MAX;
    let mut flag = LAYERED_WINDOW_ATTRIBUTES_FLAGS::default();

    unsafe {
      GetLayeredWindowAttributes(
        self.hwnd(),
        None,
        Some(&raw mut alpha),
        Some(&raw mut flag),
      )?;
    }

    if flag.contains(LWA_COLORKEY) {
      return Err(crate::Error::Platform(
        "Window uses color key for its transparency and cannot be adjusted."
          .to_string(),
      ));
    }

    let target_alpha = if opacity_delta.is_negative {
      alpha.saturating_sub(opacity_delta.inner.to_alpha())
    } else {
      alpha.saturating_add(opacity_delta.inner.to_alpha())
    };

    self.set_transparency(&OpacityValue::from_alpha(target_alpha))
  }

  pub(crate) fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()> {
    // Make the window layered if it isn't already.
    self.add_window_style_ex(WS_EX_LAYERED);

    unsafe {
      SetLayeredWindowAttributes(
        self.hwnd(),
        None,
        opacity_value.to_alpha(),
        LWA_ALPHA,
      )?;
    }

    Ok(())
  }

  /// Gets the window's position, including the window's frame. Excludes
  /// the window's shadow borders.
  pub(crate) fn frame_position(&self) -> crate::Result<Rect> {
    let mut rect = RECT::default();

    let dwm_res = unsafe {
      #[allow(clippy::cast_possible_truncation)]
      DwmGetWindowAttribute(
        self.hwnd(),
        DWMWA_EXTENDED_FRAME_BOUNDS,
        std::ptr::from_mut(&mut rect).cast(),
        std::mem::size_of::<RECT>() as u32,
      )
    };

    if let Ok(()) = dwm_res {
      Ok(Rect::from_ltrb(
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
      ))
    } else {
      warn!("Failed to get window's frame position. Falling back to border position.");
      self.border_position()
    }
  }

  /// Gets the window's position, including the window's frame and
  /// shadow borders.
  pub(crate) fn border_position(&self) -> crate::Result<Rect> {
    let mut rect = RECT::default();

    unsafe {
      GetWindowRect(self.hwnd(), std::ptr::from_mut(&mut rect).cast())
    }?;

    Ok(Rect::from_ltrb(
      rect.left,
      rect.top,
      rect.right,
      rect.bottom,
    ))
  }

  /// Gets the delta between the window's frame and the window's border.
  /// This represents the size of a window's shadow borders.
  // TODO: Return tuple of (left, top, right, bottom) instead of
  // `RectDelta`.
  pub(crate) fn shadow_borders(&self) -> crate::Result<RectDelta> {
    let border_pos = self.border_position()?;
    let frame_pos = self.frame()?;

    Ok(RectDelta::new(
      LengthValue::from_px(frame_pos.left - border_pos.left),
      LengthValue::from_px(frame_pos.top - border_pos.top),
      LengthValue::from_px(border_pos.right - frame_pos.right),
      LengthValue::from_px(border_pos.bottom - frame_pos.bottom),
    ))
  }

  fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_STYLE) };

    #[allow(clippy::cast_possible_wrap)]
    let style = style.0 as isize;
    (current_style & style) != 0
  }

  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) };

    #[allow(clippy::cast_possible_wrap)]
    let style = style.0 as isize;
    (current_style & style) != 0
  }

  pub(crate) fn restore_to_position(
    &self,
    rect: &Rect,
  ) -> crate::Result<()> {
    let placement = WINDOWPLACEMENT {
      #[allow(clippy::cast_possible_truncation)]
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

    unsafe { SetWindowPlacement(self.hwnd(), &raw const placement) }?;

    Ok(())
  }

  pub(crate) fn maximize(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_MAXIMIZE).ok() }?;
    Ok(())
  }

  pub(crate) fn minimize(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_MINIMIZE).ok() }?;
    Ok(())
  }

  pub(crate) fn close(&self) -> crate::Result<()> {
    unsafe { SendNotifyMessageW(self.hwnd(), WM_CLOSE, None, None) }?;

    Ok(())
  }

  pub(crate) fn set_visible(
    &self,
    visible: bool,
    hide_method: &HideMethod,
  ) -> crate::Result<()> {
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

  pub(crate) fn show(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_SHOWNA) }.ok()?;
    Ok(())
  }

  pub(crate) fn hide(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_HIDE) }.ok()?;
    Ok(())
  }

  pub(crate) fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    COM_INIT.with(|com_init| -> crate::Result<()> {
      let view_collection = com_init.application_view_collection()?;

      let mut view = None;
      unsafe {
        view_collection.get_view_for_hwnd(self.handle, &raw mut view)
      }
      .ok()?;

      let view = view.ok_or_else(|| {
        crate::Error::Platform(
          "Unable to get application view by window handle.".to_string(),
        )
      })?;

      // Ref: https://github.com/Ciantic/AltTabAccessor/issues/1#issuecomment-1426877843
      unsafe { view.set_cloak(1, if cloaked { 2 } else { 0 }) }
        .ok()
        .ok_or_else(|| {
          crate::Error::Platform("Failed to cloak window.".to_string())
        })
    })
  }

  /// Adds or removes the window from the native taskbar.
  ///
  /// Hidden windows (`SW_HIDE`) cannot be forced to be shown in the
  /// taskbar. Cloaked windows are normally always shown in the taskbar,
  /// but can be manually toggled.
  pub(crate) fn set_taskbar_visibility(
    &self,
    visible: bool,
  ) -> crate::Result<()> {
    COM_INIT.with(|com_init| -> crate::Result<()> {
      let taskbar_list = com_init.taskbar_list()?;

      if visible {
        unsafe { taskbar_list.AddTab(self.hwnd())? };
      } else {
        unsafe { taskbar_list.DeleteTab(self.hwnd())? };
      }

      Ok(())
    })
  }

  // TODO: Should probably be removed and have its logic called explicitly.
  pub(crate) fn set_position(
    &self,
    state: &WindowState,
    rect: &Rect,
    z_order: &ZOrder,
    is_visible: bool,
    hide_method: &HideMethod,
    has_pending_dpi_adjustment: bool,
  ) -> crate::Result<()> {
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

    let z_order = match z_order {
      ZOrder::TopMost => HWND_TOPMOST,
      ZOrder::Top => HWND_TOP,
      ZOrder::Normal => HWND_NOTOPMOST,
      ZOrder::AfterWindow(hwnd) => HWND(*hwnd),
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
            self.hwnd(),
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
            self.hwnd(),
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
              self.hwnd(),
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
    }

    // Whether to hide or show the window.
    self.set_visible(is_visible, hide_method)?;

    Ok(())
  }

  /// Marks the window as fullscreen.
  ///
  /// Causes the native Windows taskbar to be moved to the bottom of the
  /// z-order when this window is active.
  pub(crate) fn mark_fullscreen(
    &self,
    fullscreen: bool,
  ) -> crate::Result<()> {
    COM_INIT.with(|com_init| -> crate::Result<()> {
      let taskbar_list = com_init.taskbar_list()?;

      unsafe {
        taskbar_list.MarkFullscreenWindow(self.hwnd(), fullscreen)
      }?;

      Ok(())
    })
  }

  pub(crate) fn set_z_order(&self, z_order: &ZOrder) -> crate::Result<()> {
    let z_order = match z_order {
      ZOrder::TopMost => HWND_TOPMOST,
      ZOrder::Top => HWND_TOP,
      ZOrder::Normal => HWND_NOTOPMOST,
      ZOrder::AfterWindow(hwnd) => HWND(*hwnd),
    };

    unsafe {
      SetWindowPos(
        self.hwnd(),
        z_order,
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE
          | SWP_NOCOPYBITS
          | SWP_ASYNCWINDOWPOS
          | SWP_SHOWWINDOW
          | SWP_NOMOVE
          | SWP_NOSIZE,
      )
    }?;

    let handle = self.handle;

    // Z-order can sometimes still be incorrect after the above call.
    task::spawn(async move {
      tokio::time::sleep(Duration::from_millis(10)).await;

      let _ = unsafe {
        SetWindowPos(
          HWND(handle),
          z_order,
          0,
          0,
          0,
          0,
          SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_ASYNCWINDOWPOS
            | SWP_SHOWWINDOW
            | SWP_NOMOVE
            | SWP_NOSIZE,
        )
      };
    });

    Ok(())
  }

  // TODO: Should probably be removed and have its logic called explicitly.
  pub(crate) fn cleanup(&self) {
    if let Err(err) = self.show() {
      warn!("Failed to show window: {:?}", err);
    }

    _ = self.set_taskbar_visibility(true);
    _ = self.set_border_color(None);
    _ = self.set_transparency(&OpacityValue::from_alpha(u8::MAX));
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeWindow {}

impl From<NativeWindow> for crate::NativeWindow {
  fn from(window: NativeWindow) -> Self {
    crate::NativeWindow { inner: window }
  }
}

/// Windows-specific implementation of [`Dispatcher::visible_windows`].
pub(crate) fn visible_windows(
  _: &Dispatcher,
) -> crate::Result<Vec<NativeWindow>> {
  let mut handles: Vec<isize> = Vec::new();

  extern "system" fn visible_windows_proc(
    handle: HWND,
    data: LPARAM,
  ) -> BOOL {
    let handles = data.0 as *mut Vec<isize>;
    unsafe { (*handles).push(handle.0) };
    true.into()
  }

  unsafe {
    EnumWindows(
      Some(visible_windows_proc),
      LPARAM(std::ptr::from_mut(&mut handles) as _),
    )
  }?;

  Ok(
    handles
      .into_iter()
      .map(NativeWindow::new)
      .filter(|window| window.is_visible().unwrap_or(false))
      .map(Into::into)
      .collect(),
  )
}

/// Windows-specific implementation of [`Dispatcher::focused_window`].
pub(crate) fn focused_window(
  _: &Dispatcher,
) -> crate::Result<crate::NativeWindow> {
  let handle = unsafe { GetForegroundWindow() };
  Ok(NativeWindow::new(handle.0).into())
}

/// Windows-specific implementation of [`Dispatcher::window_from_point`].
pub(crate) fn window_from_point(
  point: &crate::Point,
  _: &Dispatcher,
) -> crate::Result<Option<crate::NativeWindow>> {
  let point = POINT {
    x: point.x,
    y: point.y,
  };

  let handle = unsafe { WindowFromPoint(point) };

  if handle.0 == 0 {
    return Ok(None);
  }

  Ok(Some(NativeWindow::new(handle.0).into()))
}

/// Windows-specific implementation of [`Dispatcher::reset_focus`].
pub(crate) fn reset_focus(_dispatcher: &Dispatcher) -> crate::Result<()> {
  desktop_window().focus()
}

/// Gets the `NativeWindow` instance of the desktop window.
///
/// This is the explorer.exe wallpaper window (i.e. "Progman"). If
/// explorer.exe isn't running, then default to the desktop window below
/// the wallpaper window.
#[must_use]
fn desktop_window() -> NativeWindow {
  let handle = match unsafe { GetShellWindow() } {
    HWND(0) => unsafe { GetDesktopWindow() },
    handle => handle,
  };

  NativeWindow::new(handle.0)
}
