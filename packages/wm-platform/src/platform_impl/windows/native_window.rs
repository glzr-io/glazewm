use std::time::Duration;

use tokio::task;
use tracing::warn;
use windows::{
  core::PWSTR,
  Win32::{
    Foundation::{CloseHandle, BOOL, HWND, LPARAM, POINT, RECT},
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
        EnumWindows, GetAncestor, GetClassNameW, GetDesktopWindow,
        GetForegroundWindow, GetLayeredWindowAttributes, GetShellWindow,
        GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
        SendNotifyMessageW, SetForegroundWindow,
        SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPlacement,
        SetWindowPos, ShowWindowAsync, WindowFromPoint, GA_ROOT,
        GWL_EXSTYLE, GWL_STYLE, GW_OWNER, HWND_NOTOPMOST, HWND_TOP,
        HWND_TOPMOST, LAYERED_WINDOW_ATTRIBUTES_FLAGS, LWA_ALPHA,
        LWA_COLORKEY, SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE,
        SW_SHOWNA, WINDOWPLACEMENT, WINDOW_EX_STYLE, WINDOW_STYLE,
        WM_CLOSE, WPF_ASYNCWINDOWPLACEMENT, WS_DLGFRAME, WS_EX_LAYERED,
        WS_THICKFRAME,
      },
    },
  },
};

use super::com::{ComInit, IApplicationView, COM_INIT};
use crate::{
  Color, CornerStyle, Delta, Dispatcher, LengthValue, OpacityValue, Point,
  Rect, RectDelta, WindowId, ZOrder,
};

/// Magic number used to identify programmatic mouse inputs from our own
/// process.
pub(crate) const FOREGROUND_INPUT_IDENTIFIER: u32 = 6379;

pub trait NativeWindowWindowsExt {
  /// Gets the window handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hwnd(&self) -> HWND;

  /// Gets the class name of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn class_name(&self) -> crate::Result<String>;

  /// Shows the window asynchronously.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn show(&self) -> crate::Result<()>;

  /// Hides the window asynchronously.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hide(&self) -> crate::Result<()>;

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

  /// Whether the window has an owner window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_owner_window(&self) -> bool;

  /// Whether the window has the given window style flag(s) set.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_window_style(&self, style: WINDOW_STYLE) -> bool;

  /// Whether the window has the given extended window style flag(s) set.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool;

  /// Adds the given extended window style flag(s) to the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE);

  /// Sets the window's z-order.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_z_order(&self, zorder: &ZOrder) -> crate::Result<()>;

  /// Thin wrapper around [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_window_pos(
    &self,
    z_order: &ZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()>;

  /// Cloaks or uncloaks the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_cloaked(&self, cloaked: bool) -> crate::Result<()>;

  /// Marks the window as fullscreen.
  ///
  /// Causes the native Windows taskbar to be moved to the bottom of the
  /// z-order when this window is active.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Sets the visibility of the window's title bar.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()>;

  /// Sets the color of the window's border.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()>;

  /// Sets the corner style of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()>;

  /// Sets the transparency of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()>;

  /// Adds or removes the window from the native taskbar.
  ///
  /// Cloaked windows are normally always shown in the taskbar, but can be
  /// manually toggled. Hidden windows (`SW_HIDE`) can never be shown in
  /// the taskbar.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_taskbar_visibility(&self, visible: bool) -> crate::Result<()>;

  /// Gets the delta between the window's frame and the window's border.
  /// This represents the size of a window's shadow borders.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn shadow_borders(&self) -> crate::Result<RectDelta>;

  /// Adjusts the window's transparency by a relative delta.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()>;

  /// Gets the root ancestor (top-level) window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn root_window(&self) -> crate::Result<crate::NativeWindow>;
}

impl NativeWindowWindowsExt for crate::NativeWindow {
  fn hwnd(&self) -> HWND {
    self.inner.hwnd()
  }

  fn class_name(&self) -> crate::Result<String> {
    self.inner.class_name()
  }

  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.inner.mark_fullscreen(fullscreen)
  }

  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()> {
    self.inner.set_title_bar_visibility(visible)
  }

  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()> {
    self.inner.set_border_color(color)
  }

  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()> {
    self.inner.set_corner_style(corner_style)
  }

  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()> {
    self.inner.set_transparency(opacity_value)
  }

  fn set_z_order(&self, z_order: &ZOrder) -> crate::Result<()> {
    self.inner.set_z_order(z_order)
  }

  fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()> {
    self.inner.restore(outer_frame)
  }

  fn frame_with_shadows(&self) -> crate::Result<Rect> {
    self.inner.frame_with_shadows()
  }

  fn has_owner_window(&self) -> bool {
    self.inner.has_owner_window()
  }

  fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    self.inner.has_window_style(style)
  }

  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    self.inner.has_window_style_ex(style)
  }

  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    self.inner.add_window_style_ex(style);
  }

  fn set_window_pos(
    &self,
    z_order: &ZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()> {
    self.inner.set_window_pos(z_order, rect, flags)
  }

  fn show(&self) -> crate::Result<()> {
    self.inner.show()
  }

  fn hide(&self) -> crate::Result<()> {
    self.inner.hide()
  }

  fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    self.inner.set_cloaked(cloaked)
  }

  fn set_taskbar_visibility(&self, visible: bool) -> crate::Result<()> {
    self.inner.set_taskbar_visibility(visible)
  }

  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    self.inner.shadow_borders()
  }

  fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()> {
    self.inner.adjust_transparency(opacity_delta)
  }

  fn root_window(&self) -> crate::Result<crate::NativeWindow> {
    let handle = unsafe { GetAncestor(self.inner.hwnd(), GA_ROOT) };

    if handle.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to get root ancestor window.".to_string(),
      ));
    }

    Ok(NativeWindow::new(handle.0).into())
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub(crate) handle: isize,
}

impl NativeWindow {
  /// Windows-specific implementation of [`NativeWindow::new`].
  #[must_use]
  pub(crate) fn new(handle: isize) -> Self {
    Self { handle }
  }

  /// Windows-specific implementation of [`NativeWindow::id`].
  #[must_use]
  pub(crate) fn id(&self) -> WindowId {
    WindowId(self.handle)
  }

  /// Windows-specific implementation of [`NativeWindowWindowsExt::hwnd`].
  fn hwnd(&self) -> HWND {
    HWND(self.handle)
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::has_window_style`].
  fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_STYLE) };

    #[allow(clippy::cast_possible_wrap)]
    let style = style.0 as isize;
    (current_style & style) != 0
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::add_window_style_ex`].
  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) };

    #[allow(clippy::cast_possible_wrap)]
    if current_style & style.0 as isize == 0 {
      let new_style = current_style | style.0 as isize;

      unsafe { SetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE, new_style) };
    }
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_transparency`].
  fn set_transparency(
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::class_name`].
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::mark_fullscreen`].
  pub(crate) fn mark_fullscreen(
    &self,
    fullscreen: bool,
  ) -> crate::Result<()> {
    COM_INIT.with(|com_init: &ComInit| -> crate::Result<()> {
      let taskbar_list = com_init.taskbar_list()?;

      unsafe {
        taskbar_list.MarkFullscreenWindow(self.hwnd(), fullscreen)
      }?;

      Ok(())
    })
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_title_bar_visibility`].
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_border_color`].
  pub(crate) fn set_border_color(
    &self,
    color: Option<&Color>,
  ) -> crate::Result<()> {
    let bgr = match color {
      Some(color) => color.to_bgr(),
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_corner_style`].
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_z_order`].
  pub(crate) fn set_z_order(&self, z_order: &ZOrder) -> crate::Result<()> {
    let z_order_hwnd = match z_order {
      ZOrder::TopMost => HWND_TOPMOST,
      ZOrder::Top => HWND_TOP,
      ZOrder::Normal => HWND_NOTOPMOST,
      ZOrder::AfterWindow(window_id) => HWND(window_id.0),
    };

    let flags = SWP_NOACTIVATE
      | SWP_NOCOPYBITS
      | SWP_ASYNCWINDOWPOS
      | SWP_SHOWWINDOW
      | SWP_NOMOVE
      | SWP_NOSIZE;

    unsafe { SetWindowPos(self.hwnd(), z_order_hwnd, 0, 0, 0, 0, flags) }?;

    // Z-order can sometimes still be incorrect after the above call.
    let handle = self.handle;
    task::spawn(async move {
      tokio::time::sleep(Duration::from_millis(10)).await;
      let _ = unsafe {
        SetWindowPos(HWND(handle), z_order_hwnd, 0, 0, 0, 0, flags)
      };
    });

    Ok(())
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::restore`].
  pub(crate) fn restore(
    &self,
    outer_frame: Option<&Rect>,
  ) -> crate::Result<()> {
    match outer_frame {
      None => {
        unsafe { ShowWindowAsync(self.hwnd(), SW_RESTORE) }.ok()?;
        Ok(())
      }
      Some(rect) => {
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
    }
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::has_window_style_ex`].
  pub(crate) fn has_window_style_ex(
    &self,
    style: WINDOW_EX_STYLE,
  ) -> bool {
    let current_style =
      unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) };

    #[allow(clippy::cast_possible_wrap)]
    let style = style.0 as isize;
    (current_style & style) != 0
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_window_pos`].
  pub(crate) fn set_window_pos(
    &self,
    z_order: &ZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()> {
    let z_order_hwnd = match z_order {
      ZOrder::TopMost => HWND_TOPMOST,
      ZOrder::Top => HWND_TOP,
      ZOrder::Normal => HWND_NOTOPMOST,
      ZOrder::AfterWindow(window_id) => HWND(window_id.0),
    };

    unsafe {
      SetWindowPos(
        self.hwnd(),
        z_order_hwnd,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        flags,
      )
    }?;

    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindowWindowsExt::show`].
  pub(crate) fn show(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_SHOWNA) }.ok()?;
    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindowWindowsExt::hide`].
  pub(crate) fn hide(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_HIDE) }.ok()?;
    Ok(())
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_cloaked`].
  pub(crate) fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    COM_INIT.with(|com_init: &ComInit| -> crate::Result<()> {
      let view_collection = com_init.application_view_collection()?;

      let mut view: Option<IApplicationView> = None;
      unsafe {
        view_collection.get_view_for_hwnd(self.hwnd().0, &raw mut view)
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
        .map_err(|_| {
          crate::Error::Platform("Failed to cloak window.".to_string())
        })
    })
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::set_taskbar_visibility`].
  pub(crate) fn set_taskbar_visibility(
    &self,
    visible: bool,
  ) -> crate::Result<()> {
    COM_INIT.with(|com_init: &ComInit| -> crate::Result<()> {
      let taskbar_list = com_init.taskbar_list()?;

      if visible {
        unsafe { taskbar_list.AddTab(self.hwnd())? };
      } else {
        unsafe { taskbar_list.DeleteTab(self.hwnd())? };
      }

      Ok(())
    })
  }

  /// Windows-specific implementation of [`NativeWindow::title`].
  pub(crate) fn title(&self) -> crate::Result<String> {
    let mut text: [u16; 512] = [0; 512];
    let length = unsafe { GetWindowTextW(self.hwnd(), &mut text) };

    #[allow(clippy::cast_sign_loss)]
    Ok(String::from_utf16_lossy(&text[..length as usize]))
  }

  /// Windows-specific implementation of [`NativeWindow::process_name`].
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
      let query_res = QueryFullProcessImageNameW(
        process_handle,
        PROCESS_NAME_WIN32,
        PWSTR(buffer.as_mut_ptr()),
        &raw mut length,
      );

      // Always close the process handle regardless of the query result.
      CloseHandle(process_handle)?;

      query_res
    }?;

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

  /// Windows-specific implementation of [`NativeWindow::is_visible`].
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::has_owner_window`].
  pub(crate) fn has_owner_window(&self) -> bool {
    unsafe { GetWindow(self.hwnd(), GW_OWNER) }.0 != 0
  }

  /// Windows-specific implementation of [`NativeWindow::is_minimized`].
  pub(crate) fn is_minimized(&self) -> crate::Result<bool> {
    Ok(unsafe { IsIconic(self.hwnd()) }.as_bool())
  }

  /// Windows-specific implementation of [`NativeWindow::is_maximized`].
  pub(crate) fn is_maximized(&self) -> crate::Result<bool> {
    Ok(unsafe { IsZoomed(self.hwnd()) }.as_bool())
  }

  /// Windows-specific implementation of [`NativeWindow::is_resizable`].
  pub(crate) fn is_resizable(&self) -> crate::Result<bool> {
    Ok(self.has_window_style(WS_THICKFRAME))
  }

  /// Windows-specific implementation of
  /// [`NativeWindow::is_desktop_window`].
  pub(crate) fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(*self == desktop_window())
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

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::adjust_transparency`].
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

  /// Windows-specific implementation of [`NativeWindow::frame`].
  pub(crate) fn frame(&self) -> crate::Result<Rect> {
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
      self.frame_with_shadows()
    }
  }

  /// Windows-specific implementation of
  /// [`NativeWindow::frame_with_shadows`].
  pub(crate) fn frame_with_shadows(&self) -> crate::Result<Rect> {
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

  /// Windows-specific implementation of [`NativeWindow::position`].
  pub(crate) fn position(&self) -> crate::Result<(f64, f64)> {
    let frame = self.frame()?;
    Ok((f64::from(frame.left), f64::from(frame.top)))
  }

  /// Windows-specific implementation of [`NativeWindow::size`].
  pub(crate) fn size(&self) -> crate::Result<(f64, f64)> {
    let frame = self.frame()?;
    Ok((f64::from(frame.width()), f64::from(frame.height())))
  }

  /// Windows-specific implementation of
  /// [`NativeWindowWindowsExt::shadow_borders`].
  // TODO: Return tuple of (left, top, right, bottom) instead of
  // `RectDelta`.
  pub(crate) fn shadow_borders(&self) -> crate::Result<RectDelta> {
    let border_pos = self.frame_with_shadows()?;
    let frame_pos = self.frame()?;

    Ok(RectDelta::new(
      LengthValue::from_px(frame_pos.left - border_pos.left),
      LengthValue::from_px(frame_pos.top - border_pos.top),
      LengthValue::from_px(border_pos.right - frame_pos.right),
      LengthValue::from_px(border_pos.bottom - frame_pos.bottom),
    ))
  }

  /// Windows-specific implementation of [`NativeWindow::resize`].
  pub(crate) fn resize(
    &self,
    width: i32,
    height: i32,
  ) -> crate::Result<()> {
    unsafe {
      SetWindowPos(
        self.hwnd(),
        HWND_NOTOPMOST,
        0,
        0,
        width,
        height,
        SWP_NOACTIVATE
          | SWP_NOZORDER
          | SWP_NOMOVE
          | SWP_NOCOPYBITS
          | SWP_NOSENDCHANGING
          | SWP_ASYNCWINDOWPOS
          | SWP_FRAMECHANGED,
      )
    }?;

    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindow::reposition`].
  pub(crate) fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    unsafe {
      SetWindowPos(
        self.hwnd(),
        HWND_NOTOPMOST,
        x,
        y,
        0,
        0,
        SWP_NOACTIVATE
          | SWP_NOZORDER
          | SWP_NOSIZE
          | SWP_NOCOPYBITS
          | SWP_NOSENDCHANGING
          | SWP_ASYNCWINDOWPOS
          | SWP_FRAMECHANGED,
      )
    }?;

    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindow::set_frame`].
  pub(crate) fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    unsafe {
      SetWindowPos(
        self.hwnd(),
        HWND_NOTOPMOST,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE
          | SWP_NOZORDER
          | SWP_NOCOPYBITS
          | SWP_NOSENDCHANGING
          | SWP_ASYNCWINDOWPOS
          | SWP_FRAMECHANGED,
      )
    }?;

    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindow::maximize`].
  pub(crate) fn maximize(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_MAXIMIZE).ok() }?;
    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindow::minimize`].
  pub(crate) fn minimize(&self) -> crate::Result<()> {
    unsafe { ShowWindowAsync(self.hwnd(), SW_MINIMIZE).ok() }?;
    Ok(())
  }

  /// Windows-specific implementation of [`NativeWindow::close`].
  pub(crate) fn close(&self) -> crate::Result<()> {
    unsafe { SendNotifyMessageW(self.hwnd(), WM_CLOSE, None, None) }?;
    Ok(())
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
) -> crate::Result<Vec<crate::NativeWindow>> {
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
  point: &Point,
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
