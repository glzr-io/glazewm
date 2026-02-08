use std::{
  os::windows::io::AsRawHandle,
  path::{Path, PathBuf},
  thread::JoinHandle,
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HANDLE, HWND, LPARAM, WPARAM},
    System::{
      Environment::ExpandEnvironmentStringsW, Threading::GetThreadId,
    },
    UI::{
      Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
        SHELLEXECUTEINFOW,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GetAncestor, GetMessageW,
        MessageBoxW, PeekMessageW, PostThreadMessageW, RegisterClassW,
        SystemParametersInfoW, TranslateMessage, ANIMATIONINFO, CS_HREDRAW,
        CS_VREDRAW, CW_USEDEFAULT, GA_ROOT, MB_ICONERROR, MB_OK,
        MB_SYSTEMMODAL, MSG, PM_REMOVE, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, SPI_GETANIMATION, SPI_SETANIMATION, SW_HIDE,
        SW_NORMAL, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE,
        WM_QUIT, WNDCLASSW, WNDPROC, WS_OVERLAPPEDWINDOW,
      },
    },
  },
};

use super::NativeWindow;

pub type WindowProcedure = WNDPROC;

pub struct Platform;

impl Platform {
  /// Gets a vector of available monitors as `NativeMonitor` instances
  /// sorted from left-to-right and top-to-bottom.
  ///
  /// Note that this also ensures that the `NativeMonitor` instances have
  /// valid position values.
  pub fn sorted_monitors() -> crate::Result<Vec<NativeMonitor>> {
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

  // Gets the root window of the specified window.
  pub fn root_ancestor(
    window: &NativeWindow,
  ) -> crate::Result<NativeWindow> {
    let handle = unsafe { GetAncestor(HWND(window.handle), GA_ROOT) };
    Ok(NativeWindow::new(handle.0))
  }

  /// Creates a hidden message window.
  ///
  /// Returns a handle to the created window.
  pub fn create_message_window(
    window_procedure: WindowProcedure,
  ) -> crate::Result<isize> {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("MessageWindow"),
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: window_procedure,
      ..Default::default()
    };

    unsafe { RegisterClassW(&raw const wnd_class) };

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
      return Err(crate::Error::Platform(
        "Creation of message window failed.".to_string(),
      ));
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
      if unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.as_bool() {
        unsafe {
          TranslateMessage(&raw const msg);
          DispatchMessageW(&raw const msg);
        }
      } else {
        break;
      }
    }
  }

  /// Runs a single cycle of a message loop on the current thread.
  ///
  /// Is non-blocking and returns immediately if there are no messages.
  pub fn run_message_cycle() -> crate::Result<()> {
    let mut msg = MSG::default();

    let has_message =
      unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE) }
        .as_bool();

    if has_message {
      if msg.message == WM_QUIT {
        return Err(crate::Error::Platform(
          "Received WM_QUIT message.".to_string(),
        ));
      }

      unsafe {
        TranslateMessage(&raw const msg);
        DispatchMessageW(&raw const msg);
      }
    }

    Ok(())
  }

  /// Gracefully terminates the message loop on the given thread.
  pub fn kill_message_loop<T>(
    thread: &JoinHandle<T>,
  ) -> crate::Result<()> {
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
  pub fn window_animations_enabled() -> crate::Result<bool> {
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
  pub fn set_window_animations_enabled(enable: bool) -> crate::Result<()> {
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
  pub fn open_file_explorer(path: &PathBuf) -> crate::Result<()> {
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
  /// # use wm_platform::platform::Platform;
  ///   let (prog, args) = Platform::parse_command("code .")?;
  ///   assert_eq!(prog, "code");
  ///   assert_eq!(args, ".");
  ///
  ///   let (prog, args) = Platform::parse_command(
  ///     r#"C:\Program Files\Git\git-bash --cd=C:\Users\larsb\.glaze-wm"#,
  ///   )?;
  ///   assert_eq!(prog, r#"C:\Program Files\Git\git-bash"#);
  ///   assert_eq!(args, r#"--cd=C:\Users\larsb\.glaze-wm"#);
  /// # Ok::<(), crate::Error>(())
  /// ```
  pub fn parse_command(command: &str) -> crate::Result<(String, String)> {
    // Expand environment variables in the command string.
    let expanded_command = {
      let wide_command = to_wide(command);
      let size = unsafe {
        ExpandEnvironmentStringsW(PCWSTR(wide_command.as_ptr()), None)
      };

      if size == 0 {
        return Err(crate::Error::Platform(format!(
          "Failed to expand environment strings in command '{command}'.",
        )));
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
        command.match_indices('"').nth(2).ok_or_else(|| {
          crate::Error::Platform(format!(
            "Command doesn't have an ending `\"`: '{command}'."
          ))
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

    Err(crate::Error::Platform(format!(
      "Program path is not valid for command '{command}'."
    )))
  }

  /// Runs the specified program with the given arguments.
  pub fn run_command(
    program: &str,
    args: &str,
    hide_window: bool,
  ) -> crate::Result<()> {
    let home_dir = home::home_dir()
      .ok_or_else(|| {
        crate::Error::Platform("Unable to get home directory.".to_string())
      })?
      .to_str()
      .ok_or_else(|| {
        crate::Error::Platform("Invalid home directory.".to_string())
      })?
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

    unsafe { ShellExecuteExW(&raw mut exec_info) }?;
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
