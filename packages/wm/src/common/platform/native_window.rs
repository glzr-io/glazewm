use std::sync::{Arc, RwLock};

use anyhow::Result;
use windows::{
  core::PWSTR,
  Win32::{
    Foundation::{CloseHandle, HWND},
    System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
      PROCESS_QUERY_INFORMATION,
    },
    UI::WindowsAndMessaging::{
      GetClassNameW, GetWindowTextW, GetWindowThreadProcessId,
    },
  },
};

pub type WindowHandle = HWND;

#[derive(Debug)]
pub struct NativeWindow {
  pub handle: WindowHandle,
  title: Arc<RwLock<Option<String>>>,
  process_name: Arc<RwLock<Option<String>>>,
  class_name: Arc<RwLock<Option<String>>>,
}

impl NativeWindow {
  pub fn new(handle: WindowHandle) -> Self {
    Self {
      handle,
      title: Arc::new(RwLock::new(None)),
      process_name: Arc::new(RwLock::new(None)),
      class_name: Arc::new(RwLock::new(None)),
    }
  }

  /// Gets the window's title.
  ///
  /// If the window is invalid, returns an empty string.
  pub fn title(&self) -> String {
    let title_guard = self.title.read().unwrap();
    match *title_guard {
      Some(ref title) => title.clone(),
      None => {
        let mut text: [u16; 512] = [0; 512];
        let length = unsafe { GetWindowTextW(self.handle, &mut text) };

        let title = String::from_utf16_lossy(&text[..length as usize]);
        *self.title.write().unwrap() = Some(title.clone());
        title
      }
    }
  }

  /// Gets the process name associated with the window.
  pub fn process_name(&self) -> Result<String> {
    let process_name_guard = self.process_name.read().unwrap();
    match *process_name_guard {
      Some(ref process_name) => Ok(process_name.clone()),
      None => {
        let mut process_id = 0u32;
        unsafe {
          GetWindowThreadProcessId(self.handle, Some(&mut process_id));
        }

        let process_handle = unsafe {
          OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id)
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

        let process_name = String::from_utf16(&buffer[..length as usize])?;
        *self.process_name.write().unwrap() = Some(process_name.clone());
        Ok(process_name)
      }
    }
  }

  /// Gets the class name of the window.
  pub fn class_name(&self) -> Result<String> {
    let class_name_guard = self.class_name.read().unwrap();
    match *class_name_guard {
      Some(ref class_name) => Ok(class_name.clone()),
      None => {
        let mut buffer = [0u16; 256];
        let result = unsafe { GetClassNameW(self.handle, &mut buffer) };

        if result == 0 {
          return Err(windows::core::Error::from_win32().into());
        }

        let class_name =
          String::from_utf16_lossy(&buffer[..result as usize]);
        *self.class_name.write().unwrap() = Some(class_name.clone());
        Ok(class_name)
      }
    }
  }

  pub fn is_visible(&self) -> bool {
    todo!()
  }

  pub fn is_manageable(&self) -> bool {
    todo!()
  }

  pub fn is_minimized(&self) -> bool {
    todo!()
  }

  pub fn is_maximized(&self) -> bool {
    todo!()
  }

  pub fn is_resizable(&self) -> bool {
    todo!()
  }

  pub fn is_app_bar(&self) -> bool {
    todo!()
  }

  // fn window_styles(&self) -> Vec<WindowStyle> {
  //   todo!()
  // }

  // fn window_styles_ex(&self) -> Vec<WindowStyleEx> {
  //   todo!()
  // }
}
