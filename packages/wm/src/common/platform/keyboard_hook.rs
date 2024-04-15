use std::{cell::OnceCell, collections::HashMap};

use tokio::sync::mpsc;
use windows::Win32::{
  Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
  },
};

thread_local! {
  static KEYBOARD_HOOK: OnceCell<KeyboardHook> = OnceCell::new();
}

pub struct Keybinding {
  pub key_combination: String,
  pub id: String,
}

pub struct KeyboardHook {
  hook: HHOOK,
  keybinding_map: HashMap<String, String>,
  sender: mpsc::Sender<String>,
  pub receiver: mpsc::Receiver<String>,
}

impl KeyboardHook {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel(32);

    Self {
      hook: HHOOK(0),
      keybinding_map: HashMap::new(),
      sender,
      receiver,
    }
  }

  pub fn start(
    &mut self,
    keybindings: Vec<Keybinding>,
  ) -> anyhow::Result<()> {
    self.add_keybindings(keybindings);

    // Register low-level keyboard hook.
    unsafe {
      self.hook = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        Some(keyboard_hook_proc),
        HINSTANCE(0),
        0,
      )?;
    };

    Ok(())
  }

  pub fn update(&mut self, keybindings: Vec<Keybinding>) {
    self.keybinding_map.clear();
    self.add_keybindings(keybindings);
  }

  pub fn stop(&mut self) {
    self.keybinding_map.clear();
    let _ = unsafe { UnhookWindowsHookEx(self.hook) };
  }

  fn add_keybindings(&mut self, keybindings: Vec<Keybinding>) {
    for keybinding in keybindings {
      self
        .keybinding_map
        .insert(keybinding.key_combination, keybinding.id);
    }
  }

  fn handle_key_event(&self, key_combination: &str) {
    if let Some(id) = self.keybinding_map.get(key_combination) {
      let _ = self.sender.try_send(id.clone());
    }
  }
}

extern "system" fn keyboard_hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if code >= 0 {
    let kb = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let key_combination = format!("key_{}", kb.vkCode);

    // TODO: Implement proper key combination parsing and matching
    KEYBOARD_HOOK.with(|hook| {
      if let Some(hook) = hook.get() {
        hook.handle_key_event(&key_combination);
      }
    });
  }

  unsafe { CallNextHookEx(HHOOK(0), code, wparam, lparam) }
}
