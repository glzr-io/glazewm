use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use tokio::sync::oneshot;
use tracing::info;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, LocalFree};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
use windows::Win32::System::Threading::DETACHED_PROCESS;
use windows::Win32::System::Threading::{
  CreateProcessW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION,
  STARTUPINFOW,
};
use windows::Win32::UI::Shell::ShellExecuteExW;
use windows::Win32::UI::Shell::SEE_MASK_NOASYNC;
use windows::Win32::UI::Shell::SEE_MASK_NOCLOSEPROCESS;
use windows::Win32::UI::Shell::SHELLEXECUTEINFOW;
use windows::Win32::UI::Shell::{
  CommandLineToArgvW, SHEvaluateSystemCommandTemplate,
};
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

pub fn shell_exec(command: &str) -> anyhow::Result<()> {
  // TODO: Use SHExpandEnvironmentStrings to expand environment variables.

  // Arbitrary command to execute.
  // let command = r#""C:\Program Files\Git\git-bash" --cd="./bin""#;
  // let command = r#"%ProgramFiles%\Git\git-bash" --cd="./bin""#;
  let (prog, mut args) = command_template4(command)?;
  // let (prog, mut args) = command_template2(command)?;

  // let window_thread = std::thread::spawn(move || {
  //   // let (abort_tx, abort_rx) = oneshot::channel();
  //   // unsafe { Platform::create_message_loop(abort_rx, None) };

  //   // let output = Command::new(prog).args(args.split(" ")).spawn();
  // let output = Command::new(prog)
  //   .args(args.split(" "))
  //   .creation_flags(DETACHED_PROCESS.0 | CREATE_NEW_PROCESS_GROUP.0)
  //   .spawn();

  // .context("Failed to execute command");
  // });

  // let mut exec_info: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW {
  //   cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
  //   lpVerb: PWSTR(verb.as_mut_ptr()),
  //   lpFile: PWSTR(file.as_mut_ptr()),
  //   lpDirectory: PWSTR(directory.as_mut_ptr()),
  //   lpParameters: PWSTR(parameters.as_mut_ptr()),
  //   nShow: 1,
  //   fMask: 0,
  //   hwnd: HWND(0),
  //   hInstApp: HINSTANCE(0),
  //   hkeyClass: HKEY(0),
  //   lpClass: PWSTR(class.as_mut_ptr()),
  //   Anonymous: SHELLEXECUTEINFOW_0 { hIcon: HANDLE(0) },
  //   ..Default::default()
  // };

  let home_dir =
    home::home_dir().context("Unable to get home directory.")?;

  let home_dir_str =
    home_dir.to_str().context("Invalid home directory.")?;

  // TODO: With the current `SHELLEXECUTEINFOW` struct, the opened
  // application can take over the console, and/or fully block the
  // the current process until it exits.
  let mut exec_info = SHELLEXECUTEINFOW {
    cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
    // lpVerb: None,
    lpFile: PCWSTR(prog.as_ptr()),
    lpParameters: PCWSTR(args.as_ptr()),
    // lpDirectory: PCWSTR(std::ptr::null()),
    lpDirectory: PCWSTR(to_wide(home_dir_str).as_ptr()),
    nShow: SW_NORMAL.0 as _,
    // fMask: SEE_MASK_NOASYNC,
    fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
    ..Default::default()
  };

  unsafe {
    ShellExecuteExW(&mut exec_info)?;
    WaitForSingleObject(exec_info.hProcess, u32::MAX);
    let _ = CloseHandle(exec_info.hProcess)?;
  };

  // let mut startup_info = STARTUPINFOW::default();
  // let mut process_info = PROCESS_INFORMATION::default();

  // // Using the built-in `Command::new` function in Rust causes some
  // // applications to instantly exit.
  // unsafe {
  //   CreateProcessW(
  //     PCWSTR(prog.as_ptr()),
  //     PWSTR(args.as_mut_ptr()),
  //     None,
  //     None,
  //     false,
  //     PROCESS_CREATION_FLAGS::default(),
  //     None,
  //     None,
  //     &mut startup_info,
  //     &mut process_info,
  //   )?;

  info!("Command executed successfully: {}.", command);
  //   CloseHandle(process_info.hProcess)?;
  //   CloseHandle(process_info.hThread)?;
  // }

  Ok(())
}

fn to_wide(s: &str) -> Vec<u16> {
  OsString::from(s).encode_wide().chain(Some(0)).collect()
}

fn command_template4(command: &str) -> anyhow::Result<(PWSTR, PWSTR)> {
  let mut application: PWSTR = PWSTR::null();
  let mut parameters: PWSTR = PWSTR::null();

  unsafe {
    SHEvaluateSystemCommandTemplate(
      PCWSTR(to_wide(&command).as_ptr()),
      &mut application,
      None,
      Some(&mut parameters),
    )
  }
  .with_context(|| {
    format!("Program path is not valid for command '{}'.", command)
  })?;

  let application_str = unsafe { application.to_string()? };
  let parameters_str = unsafe { parameters.to_string()? };

  info!(
    "Parsed command program: '{}', args: '{}'.",
    application_str, parameters_str
  );

  // Free the memory allocated by `SHEvaluateSystemCommandTemplate`.
  // unsafe { CoTaskMemFree(Some(application.0 as _)) };
  // unsafe { CoTaskMemFree(Some(parameters.0 as _)) }

  // let application_str = unsafe { application.to_string()? };
  // let parameters_str = unsafe { parameters.to_string()? };

  Ok((application, parameters))
}

fn command_template3(
  command: &str,
) -> anyhow::Result<(Vec<u16>, Vec<u16>)> {
  // let command_utf16: Vec<u16> =
  //   command.encode_utf16().chain(Some(0)).collect();
  // let command_utf16 = to_wide(&command);
  info!("asdjfoiasdjfio",);

  let mut application: PWSTR = PWSTR::null();
  let mut parameters: PWSTR = PWSTR::null();

  unsafe {
    SHEvaluateSystemCommandTemplate(
      PCWSTR(to_wide(&command).as_ptr()),
      &mut application,
      None,
      Some(&mut parameters),
    )
  }
  .with_context(|| {
    format!("Program path is not valid for command '{}'.", command)
  })?;

  let application_str = unsafe { application.to_string()? };
  let parameters_str = unsafe { parameters.to_string()? };

  info!(
    "Parsed command program: '{}', args: '{}'.",
    application_str, parameters_str
  );

  // Free the memory allocated by `SHEvaluateSystemCommandTemplate`.
  unsafe { CoTaskMemFree(Some(application.0 as _)) };
  unsafe { CoTaskMemFree(Some(parameters.0 as _)) }

  // let application_str = unsafe { application.to_string()? };
  // let parameters_str = unsafe { parameters.to_string()? };

  Ok((to_wide(&application_str), to_wide(&parameters_str)))
}

fn command_template2(command: &str) -> anyhow::Result<((String, String))> {
  let command_utf16: Vec<u16> =
    command.encode_utf16().chain(Some(0)).collect();

  let mut application: PWSTR = PWSTR::null();
  let mut command_line: PWSTR = PWSTR::null();
  let mut parameters: PWSTR = PWSTR::null();

  unsafe {
    SHEvaluateSystemCommandTemplate(
      PCWSTR(command_utf16.as_ptr()),
      &mut application,
      Some(&mut command_line),
      Some(&mut parameters),
    )
  }?;

  let application_str =
    unsafe { application.to_string().unwrap_or_default() };
  let command_line_str =
    unsafe { command_line.to_string().unwrap_or_default() };
  let parameters_str =
    unsafe { parameters.to_string().unwrap_or_default() };

  println!("Application: {}", application_str);
  println!("Command Line: {}", command_line_str);
  println!("Parameters: {}", parameters_str);

  // Free the memory allocated by SHEvaluateSystemCommandTemplate
  unsafe {
    windows::Win32::System::Com::CoTaskMemFree(Some(application.0 as _))
  };
  unsafe {
    windows::Win32::System::Com::CoTaskMemFree(Some(command_line.0 as _))
  };
  unsafe {
    windows::Win32::System::Com::CoTaskMemFree(Some(parameters.0 as _))
  }

  Ok((application_str, parameters_str))
}

fn command_template(buf: &[u16]) -> anyhow::Result<()> {
  let mut ppszapplication = [0u16; 256];
  let mut ppszcommandline = [0u16; 256];
  let mut ppszparameters = [0u16; 256];
  unsafe {
    SHEvaluateSystemCommandTemplate(
      PCWSTR(buf.as_ptr()),
      &mut PWSTR(ppszapplication.as_mut_ptr()),
      Some(&mut PWSTR(ppszcommandline.as_mut_ptr())),
      Some(&mut PWSTR(ppszparameters.as_mut_ptr())),
    )
  }?;

  println!("Application: {:?}", ppszapplication);
  println!("Application: {:?}", ppszcommandline);
  println!("Application: {:?}", ppszparameters);

  Ok(())
}
