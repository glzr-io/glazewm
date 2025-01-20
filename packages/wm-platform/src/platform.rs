use std::{
  os::windows::io::AsRawHandle,
  path::{Path, PathBuf},
  thread::JoinHandle,
};

use anyhow::{bail, Context};
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HANDLE, HWND, LPARAM, POINT, WPARAM},
    System::{
      Environment::ExpandEnvironmentStringsW, Threading::GetThreadId,
    },
    UI::{
      Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
        SHELLEXECUTEINFOW,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GetAncestor, GetCursorPos,
        GetDesktopWindow, GetForegroundWindow, GetMessageW,
        GetShellWindow, MessageBoxW, PeekMessageW, PostThreadMessageW,
        RegisterClassW, SetCursorPos, SystemParametersInfoW,
        TranslateMessage, WindowFromPoint, ANIMATIONINFO, CS_HREDRAW,
        CS_VREDRAW, CW_USEDEFAULT, GA_ROOT, MB_ICONERROR, MB_OK,
        MB_SYSTEMMODAL, MSG, PM_REMOVE, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, SPI_GETANIMATION, SPI_SETANIMATION, SW_HIDE,
        SW_NORMAL, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE,
        WM_QUIT, WNDCLASSW, WNDPROC, WS_OVERLAPPEDWINDOW,
      },
    },
  },
};
use wm_common::{ParsedConfig, Point};

use super::{
  native_monitor, native_window, EventListener, NativeMonitor,
  NativeWindow, SingleInstance,
};

pub type WindowProcedure = WNDPROC;

pub struct Platform;

impl Platform {
  /// Gets the `NativeWindow` instance of the currently focused window.
  #[must_use]
  pub fn foreground_window() -> NativeWindow {
    let handle = unsafe { GetForegroundWindow() };
    NativeWindow::new(handle.0)
  }

  /// Gets the `NativeWindow` instance of the desktop window.
  ///
  /// This is the explorer.exe wallpaper window (i.e. "Progman"). If
  /// explorer.exe isn't running, then default to the desktop window below
  /// the wallpaper window.
  #[must_use]
  pub fn desktop_window() -> NativeWindow {
    let handle = match unsafe { GetShellWindow() } {
      HWND(0) => unsafe { GetDesktopWindow() },
      handle => handle,
    };

    NativeWindow::new(handle.0)
  }

  /// Gets a vector of available monitors as `NativeMonitor` instances
  /// sorted from left-to-right and top-to-bottom.
  ///
  /// Note that this also ensures that the `NativeMonitor` instances have
  /// valid position values.
  pub fn sorted_monitors() -> anyhow::Result<Vec<NativeMonitor>> {
    let monitors = native_monitor::available_monitors()?;

    // Create a tuple of monitors and their rects.
    let mut monitors_with_rect = monitors
      .into_iter()
      .map(|monitor| {
        let rect = monitor.rect()?.clone();
        anyhow::Ok((monitor, rect))
      })
      .try_collect::<Vec<_>>()?;

    // Sort monitors from left-to-right, top-to-bottom.
    monitors_with_rect.sort_by(|(_, rect_a), (_, rect_b)| {
      if rect_a.x() == rect_b.x() {
        rect_a.y().cmp(&rect_b.y())
      } else {
        rect_a.x().cmp(&rect_b.x())
      }
    });

    // Convert back to a regular vector of monitors.
    Ok(
      monitors_with_rect
        .into_iter()
        .map(|(monitor, _)| monitor)
        .collect(),
    )
  }

  #[must_use]
  pub fn nearest_monitor(window: &NativeWindow) -> NativeMonitor {
    native_monitor::nearest_monitor(window.handle)
  }

  /// Gets a vector of "manageable" windows as `NativeWindow` instances.
  ///
  /// Manageable windows are visible windows that the WM is most likely
  /// able to manage. Windows are returned in z-order (top to bottom),
  /// although the order is not guaranteed by the underlying API.
  pub fn manageable_windows() -> anyhow::Result<Vec<NativeWindow>> {
    Ok(
      native_window::available_windows()?
        .into_iter()
        .filter(|window| window.is_manageable().unwrap_or(false))
        .collect(),
    )
  }

  /// Creates a new `EventListener` for the specified user config.
  pub fn start_event_listener(
    config: &ParsedConfig,
  ) -> anyhow::Result<EventListener> {
    EventListener::start(config)
  }

  /// Creates a new `SingleInstance`.
  pub fn new_single_instance() -> anyhow::Result<SingleInstance> {
    SingleInstance::new()
  }

  // Gets the root window of the specified window.
  pub fn root_ancestor(
    window: &NativeWindow,
  ) -> anyhow::Result<NativeWindow> {
    let handle = unsafe { GetAncestor(HWND(window.handle), GA_ROOT) };
    Ok(NativeWindow::new(handle.0))
  }

  /// Sets the cursor position to the specified coordinates.
  pub fn set_cursor_pos(x: i32, y: i32) -> anyhow::Result<()> {
    unsafe {
      SetCursorPos(x, y)?;
    };

    Ok(())
  }

  /// Finds the window at the specified point in screen space.
  pub fn window_from_point(point: &Point) -> anyhow::Result<NativeWindow> {
    let point = POINT {
      x: point.x,
      y: point.y,
    };

    let handle = unsafe { WindowFromPoint(point) };
    Ok(NativeWindow::new(handle.0))
  }

  /// Gets the mouse position in screen space.
  pub fn mouse_position() -> anyhow::Result<Point> {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut point) }?;

    Ok(Point {
      x: point.x,
      y: point.y,
    })
  }

  /// Creates a hidden message window.
  ///
  /// Returns a handle to the created window.
  pub fn create_message_window(
    window_procedure: WindowProcedure,
  ) -> anyhow::Result<isize> {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("MessageWindow"),
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: window_procedure,
      ..Default::default()
    };

    unsafe { RegisterClassW(&wnd_class) };

    let handle = unsafe {
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("MessageWindow"),
        w!("MessageWindow"),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        None,
        None,
        wnd_class.hInstance,
        None,
      )
    };

    if handle.0 == 0 {
      bail!("Creation of message window failed.");
    }

    Ok(handle.0)
  }

  /// Starts a message loop on the current thread.
  ///
  /// This function will block until the message loop is killed. Use
  /// `Platform::kill_message_loop` to terminate the message loop.
  pub fn run_message_loop() {
    let mut msg = MSG::default();

    loop {
      if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        unsafe {
          TranslateMessage(&msg);
          DispatchMessageW(&msg);
        }
      } else {
        break;
      }
    }
  }

  /// Runs a single cycle of a message loop on the current thread.
  ///
  /// Is non-blocking and returns immediately if there are no messages.
  pub fn run_message_cycle() -> anyhow::Result<()> {
    let mut msg = MSG::default();

    let has_message =
      unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool();

    if has_message {
      if msg.message == WM_QUIT {
        bail!("Received WM_QUIT message.")
      }

      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
    }

    Ok(())
  }

  /// Gracefully terminates the message loop on the given thread.
  pub fn kill_message_loop<T>(
    thread: &JoinHandle<T>,
  ) -> anyhow::Result<()> {
    let handle = thread.as_raw_handle();
    let handle = HANDLE(handle as isize);
    let thread_id = unsafe { GetThreadId(handle) };

    unsafe {
      PostThreadMessageW(
        thread_id,
        WM_QUIT,
        WPARAM::default(),
        LPARAM::default(),
      )
    }?;

    Ok(())
  }

  /// Gets whether window transition animations are currently enabled.
  ///
  /// Note that this is a global system setting.
  pub fn window_animations_enabled() -> anyhow::Result<bool> {
    let mut animation_info = ANIMATIONINFO {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32,
      iMinAnimate: 0,
    };

    unsafe {
      SystemParametersInfoW(
        SPI_GETANIMATION,
        animation_info.cbSize,
        Some(std::ptr::from_mut(&mut animation_info).cast()),
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
      )
    }?;

    Ok(animation_info.iMinAnimate != 0)
  }

  /// Enables or disables window transition animations.
  ///
  /// Note that this is a global system setting.
  pub fn set_window_animations_enabled(
    enable: bool,
  ) -> anyhow::Result<()> {
    let mut animation_info = ANIMATIONINFO {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32,
      iMinAnimate: i32::from(enable),
    };

    unsafe {
      SystemParametersInfoW(
        SPI_SETANIMATION,
        animation_info.cbSize,
        Some(std::ptr::from_mut(&mut animation_info).cast()),
        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
      )
    }?;

    Ok(())
  }

  /// Opens File Explorer at the specified path.
  pub fn open_file_explorer(path: &PathBuf) -> anyhow::Result<()> {
    let normalized_path = std::fs::canonicalize(path)?;

    std::process::Command::new("explorer")
      .arg(normalized_path)
      .spawn()?;

    Ok(())
  }

  /// Parses a command string into a program name/path and arguments. This
  /// also expands any environment variables found in the command string if
  /// they are wrapped in `%` characters. If the command string is a path,
  /// a file extension is required.
  ///
  /// This is similar to the `SHEvaluateSystemCommandTemplate` function. It
  /// also parses program name/path and arguments, but can't handle `/` as
  /// file path delimiters and it errors for certain programs (e.g.
  /// `code`).
  ///
  /// Returns a tuple containing the program name/path and arguments.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use wm::common::platform::Platform;
  ///   let (prog, args) = Platform::parse_command("code .")?;
  ///   assert_eq!(prog, "code");
  ///   assert_eq!(args, ".");
  ///
  ///   let (prog, args) = Platform::parse_command(
  ///     r#"C:\Program Files\Git\git-bash --cd=C:\Users\larsb\.glaze-wm"#,
  ///   )?;
  ///   assert_eq!(prog, r#"C:\Program Files\Git\git-bash"#);
  ///   assert_eq!(args, r#"--cd=C:\Users\larsb\.glaze-wm"#);
  /// # Ok::<(), anyhow::Error>(())
  /// ```
  pub fn parse_command(command: &str) -> anyhow::Result<(String, String)> {
    // Expand environment variables in the command string.
    let expanded_command = {
      let wide_command = to_wide(command);
      let size = unsafe {
        ExpandEnvironmentStringsW(PCWSTR(wide_command.as_ptr()), None)
      };

      if size == 0 {
        anyhow::bail!(
          "Failed to expand environment strings in command '{}'.",
          command
        );
      }

      let mut buffer = vec![0; size as usize];
      let size = unsafe {
        ExpandEnvironmentStringsW(
          PCWSTR(wide_command.as_ptr()),
          Some(&mut buffer),
        )
      };

      // The size includes the null terminator, so we need to subtract one.
      String::from_utf16_lossy(&buffer[..(size - 1) as usize])
    };

    let command_parts: Vec<&str> =
      expanded_command.split_whitespace().collect();

    // If the command starts with double quotes, then the program name/path
    // is wrapped in double quotes (e.g. `"C:\path\to\app.exe" --flag`).
    if command.starts_with('"') {
      // Find the closing double quote.
      let (closing_index, _) =
        command.match_indices('"').nth(2).with_context(|| {
          format!("Command doesn't have an ending `\"`: '{command}'.")
        })?;

      return Ok((
        command[1..closing_index].to_string(),
        command[closing_index + 1..].trim().to_string(),
      ));
    }

    // The first part is the program name if it doesn't contain a slash or
    // backslash.
    if let Some(first_part) = command_parts.first() {
      if !first_part.contains(&['/', '\\'][..]) {
        let args = command_parts[1..].join(" ");
        return Ok(((*first_part).to_string(), args));
      }
    }

    let mut cumulative_path = Vec::new();

    // Lastly, iterate over the command until a valid file path is found.
    for (part_index, &part) in command_parts.iter().enumerate() {
      cumulative_path.push(part);

      if Path::new(&cumulative_path.join(" ")).is_file() {
        return Ok((
          cumulative_path.join(" "),
          command_parts[part_index + 1..].join(" "),
        ));
      }
    }

    anyhow::bail!("Program path is not valid for command '{}'.", command)
  }

  /// Runs the specified program with the given arguments.
  pub fn run_command(
    program: &str,
    args: &str,
    hide_window: bool,
  ) -> anyhow::Result<()> {
    let home_dir = home::home_dir()
      .context("Unable to get home directory.")?
      .to_str()
      .context("Invalid home directory.")?
      .to_owned();

    // Inlining the wide variables within the `SHELLEXECUTEINFOW` struct
    // causes issues where the pointer is dropped while `ShellExecuteExW`
    // is using it. This is likely a `windows-rs` bug, and we can avoid
    // it by keeping separate variables for the wide strings.
    let program_wide = to_wide(program);
    let args_wide = to_wide(args);
    let home_dir_wide = to_wide(&home_dir);

    // Using the built-in `Command::new` function in Rust launches the
    // program as a subprocess. This prevents Windows from cleaning up
    // handles held by our process (e.g. the IPC server port) until the
    // subprocess exits.
    let mut exec_info = SHELLEXECUTEINFOW {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
      lpFile: PCWSTR(program_wide.as_ptr()),
      lpParameters: PCWSTR(args_wide.as_ptr()),
      lpDirectory: PCWSTR(home_dir_wide.as_ptr()),
      nShow: if hide_window { SW_HIDE } else { SW_NORMAL }.0 as _,
      fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
      ..Default::default()
    };

    unsafe { ShellExecuteExW(&mut exec_info) }?;
    Ok(())
  }

  pub fn show_error_dialog(title: &str, message: &str) {
    let title_wide = to_wide(title);
    let message_wide = to_wide(message);

    unsafe {
      MessageBoxW(
        None,
        PCWSTR(message_wide.as_ptr()),
        PCWSTR(title_wide.as_ptr()),
        MB_ICONERROR | MB_OK | MB_SYSTEMMODAL,
      );
    }
  }
}

/// Utility function to convert a string to a null-terminated wide string.
fn to_wide(string: &str) -> Vec<u16> {
  string.encode_utf16().chain(Some(0)).collect()
}
