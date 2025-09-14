use super::ax_ui_element::AXUIElementRef;

pub type ProcessId = i32;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
  pub fn AXUIElementCreateApplication(pid: ProcessId) -> AXUIElementRef;
}
