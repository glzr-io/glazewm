use std::path::Path;
use std::sync::Arc;

use crate::common::Point;
use anyhow::{bail, Context};
use tokio::sync::{oneshot, Mutex};
use tracing::warn;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
use windows::Win32::UI::Shell::{
  ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
  SHELLEXECUTEINFOW,
};

use windows::Win32::UI::WindowsAndMessaging::GA_ROOT;
use windows::Win32::UI::{
  HiDpi::{
    SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
  },
  WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, GetAncestor,
    GetDesktopWindow, GetForegroundWindow, GetMessageW, GetParent,
    RegisterClassW, SetCursorPos, TranslateMessage, WindowFromPoint,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, SW_NORMAL, WNDCLASSW,
    WNDPROC, WS_OVERLAPPEDWINDOW,
  },
};

use crate::user_config::UserConfig;

use super::{
  native_monitor, native_window, EventListener, NativeMonitor,
  NativeWindow, SingleInstance,
};

pub type WindowProcedure = WNDPROC;

pub struct Platform;

impl Platform {
  /// Gets the `NativeWindow` instance of the currently focused window.
  pub fn foreground_window() -> NativeWindow {
    let handle = unsafe { GetForegroundWindow() };
    NativeWindow::new(handle.0)
  }

  /// Gets the `NativeWindow` instance of the desktop window.
  pub fn desktop_window() -> NativeWindow {
    let handle = unsafe { GetDesktopWindow() };
    NativeWindow::new(handle.0)
  }

  /// Gets a vector of available monitors as `NativeMonitor` instances
  /// sorted from left-to-right and top-to-bottom.
  ///
  /// Note that this also ensures that the `NativeMonitor` instances have
  /// valid position values.
  pub fn sorted_monitors() -> anyhow::Result<Vec<NativeMonitor>> {
    let monitors = native_monitor::available_monitors()?;

    // Create a tuple of monitor and its rect.
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

  pub fn nearest_monitor(window: &NativeWindow) -> NativeMonitor {
    native_monitor::nearest_monitor(window.handle)
  }

  /// Gets a vector of "manageable" windows as `NativeWindow` instances.
  ///
  /// Manageable windows are visible windows that the WM is most likely
  /// able to manage.
  pub fn manageable_windows() -> anyhow::Result<Vec<NativeWindow>> {
    Ok(
      native_window::available_windows()?
        .into_iter()
        .filter(|w| w.is_manageable())
        .collect(),
    )
  }

  /// Creates a new `EventListener` for the specified user config.
  pub fn start_event_listener(
    config: &UserConfig,
  ) -> anyhow::Result<EventListener> {
    EventListener::start(config)
  }

  /// Creates a new `SingleInstance`.
  pub fn new_single_instance() -> anyhow::Result<SingleInstance> {
    SingleInstance::new()
  }

  // Gets the root window of the specified window.
  pub fn get_root_ancestor(
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

  /// Sets the DPI awareness for the current process to per-monitor
  /// awareness (v2).
  pub fn set_dpi_awareness() -> anyhow::Result<()> {
    unsafe {
      SetProcessDpiAwarenessContext(
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
      )
    }?;

    Ok(())
  }

  // Find the window at the specified point in screen space.
  pub fn window_from_point(point: &Point) -> anyhow::Result<NativeWindow> {
    let point = POINT {
      x: point.x,
      y: point.y,
    };

    let handle = unsafe { WindowFromPoint(point) };
    Ok(NativeWindow::new(handle.0))
  }

  /// Spawns a hidden message window and starts a message loop.
  ///
  /// This function will block until the message loop is aborted via the
  /// `abort_rx` channel.
  pub unsafe fn create_message_loop(
    mut abort_rx: oneshot::Receiver<()>,
    window_procedure: WindowProcedure,
  ) -> anyhow::Result<isize> {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("MessageWindow"),
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: window_procedure,
      ..Default::default()
    };

    RegisterClassW(&wnd_class);

    let handle = CreateWindowExW(
      Default::default(),
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
    );

    if handle.0 == 0 {
      bail!("Creation of message window failed.");
    }

    let mut msg = MSG::default();

    loop {
      // Check whether the abort signal has been received.
      if abort_rx.try_recv().is_ok() {
        if let Err(err) = DestroyWindow(handle) {
          warn!("Failed to destroy message window '{}'.", err);
        }
        break;
      }

      if GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      } else {
        break;
      }
    }

    Ok(handle.0)
  }

  /// Parses a command string into a program name/path and arguments. This
  /// also expands any environment variables found in the command string if
  /// they are wrapped in `%` characters. If the command string is a path, a
  /// file extension is required.
  ///
  /// This is similar to the `SHEvaluateSystemCommandTemplate` function. It
  /// also parses program name/path and arguments, but can't handle `/` as
  /// file path delimiters and it errors for certain programs (e.g. `code`).
  ///
  /// Returns a tuple containing the program name/path and arguments.
  ///
  /// # Examples
  ///
  /// ```
  /// let (prog, args) = parse_command("code .")?;
  /// assert_eq!(prog, "code");
  /// assert_eq!(args, ".");
  ///
  /// let (prog, args) = parse_command(
  ///   r#"C:\Program Files\Git\git-bash --cd=C:\Users\larsb\.glaze-wm"#,
  /// )?;
  /// assert_eq!(prog, r#"C:\Program Files\Git\git-bash"#);
  /// assert_eq!(args, r#"--cd=C:\Users\larsb\.glaze-wm"#);
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
      expanded_command.trim().split_whitespace().collect();

    // If the command starts with double quotes, then the program name/path
    // is wrapped in double quotes (e.g. `"C:\path\to\app.exe" --flag`).
    if command.starts_with("\"") {
      // Find the closing double quote.
      let (closing_index, _) =
        command.match_indices('"').nth(2).with_context(|| {
          format!("Command doesn't have an ending `\"`: '{}'.", command)
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
        return Ok((first_part.to_string(), args));
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
  pub fn run_command(program: &str, args: &str) -> anyhow::Result<()> {
    let home_dir = home::home_dir()
      .context("Unable to get home directory.")?
      .to_str()
      .context("Invalid home directory.")?
      .to_owned();

    // Inlining the wide variables within the `SHELLEXECUTEINFOW` struct
    // causes issues where the pointer is dropped while `ShellExecuteExW` is
    // using it. This is likely a `windows-rs` bug, and we can avoid it by
    // keeping separate variables for the wide strings.
    let program_wide = to_wide(&program);
    let args_wide = to_wide(&args);
    let home_dir_wide = to_wide(&home_dir);

    // Using the built-in `Command::new` function in Rust launches the
    // program as a subprocess. This prevents Windows from cleaning up
    // handles held by our process (e.g. the IPC server port) until the
    // subprocess exits.
    let mut exec_info = SHELLEXECUTEINFOW {
      cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
      lpFile: PCWSTR(program_wide.as_ptr()),
      lpParameters: PCWSTR(args_wide.as_ptr()),
      lpDirectory: PCWSTR(home_dir_wide.as_ptr()),
      nShow: SW_NORMAL.0 as _,
      fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
      ..Default::default()
    };

    unsafe { ShellExecuteExW(&mut exec_info) }?;
    Ok(())
  }
}

/// Utility function to convert a string to a null-terminated wide string.
fn to_wide(string: &str) -> Vec<u16> {
  string.encode_utf16().chain(Some(0)).collect()
}
