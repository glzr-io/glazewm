use std::cell::RefCell;

use anyhow::Result;

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

  // fn title(&self) -> Result<String> {
  //   match self.title {
  //     Some(title) => Ok(title.borrow().clone()),
  //     None => {
  //       let title = windows_api::window_title_by_handle()?;
  //       self.title = Some(RefCell::new(title));
  //       Ok(title)
  //     }
  //   }
  // }

  // fn process_name(&self) -> Result<String> {
  //   match self.process_name {
  //     Some(process_name) => Ok(process_name.borrow().clone()),
  //     None => {
  //       let process_name = windows_api::process_name_by_handle()?;
  //       self.process_name = Some(RefCell::new(process_name));
  //       Ok(process_name)
  //     }
  //   }
  // }

  // fn class_name(&self) -> Result<String> {
  //   match self.class_name {
  //     Some(class_name) => Ok(class_name.borrow().clone()),
  //     None => {
  //       let class_name = windows_api::class_name_by_handle()?;
  //       self.class_name = Some(RefCell::new(class_name));
  //       Ok(class_name)
  //     }
  //   }
  // }

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
