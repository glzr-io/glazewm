use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{ShowWindowAsync, SW_MINIMIZE},
};

pub fn cleanup_windows(managed_handles: Vec<isize>) {
  managed_handles.iter().for_each(|handle| unsafe {
    ShowWindowAsync(HWND(*handle), SW_MINIMIZE);
  });
}
