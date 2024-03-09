use std::cell::RefCell;

use anyhow::Result;
use windows::{
  core::PWSTR,
  Win32::{
    Foundation::CloseHandle,
    System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
      PROCESS_QUERY_INFORMATION,
    },
    UI::WindowsAndMessaging::{
      GetClassNameW, GetWindowTextW, GetWindowThreadProcessId,
    },
  },
};

use super::WindowHandle;

#[derive(Debug)]
pub struct NativeWindow {
  pub handle: WindowHandle,
  title: Option<RefCell<String>>,
  process_name: Option<RefCell<String>>,
  class_name: Option<RefCell<String>>,
}

impl NativeWindow {
  pub fn new(handle: WindowHandle) -> Self {
    Self {
      handle,
      title: None,
      process_name: None,
      class_name: None,
    }
  }

  /// Gets the window title.
  ///
  /// If the window is invalid, returns an empty string.
  fn title(&self) -> String {
    match self.title {
      Some(title) => title.borrow().clone(),
      None => {
        let mut text: [u16; 512] = [0; 512];
        let length = unsafe { GetWindowTextW(self.handle, &mut text) };

        let title = String::from_utf16_lossy(&text[..length as usize]);
        self.title = Some(RefCell::new(title));
        title
      }
    }
  }

  fn process_name(&self) -> Result<String> {
    match self.process_name {
      Some(process_name) => Ok(process_name.borrow().clone()),
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
        self.process_name = Some(RefCell::new(process_name));
        Ok(process_name)
      }
    }
  }

  fn class_name(&self) -> Result<String> {
    match self.class_name {
      Some(class_name) => Ok(class_name.borrow().clone()),
      None => {
        let mut buffer = [0u16; 256];
        let result = unsafe { GetClassNameW(self.handle, &mut buffer) };

        if result == 0 {
          return Err(windows::core::Error::from_win32().into());
        }

        let class_name =
          String::from_utf16_lossy(&buffer[..result as usize]);
        println!("Class name: {}", class_name);
        self.class_name = Some(RefCell::new(class_name));
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
