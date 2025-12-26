use std::{
    sync::{Arc, Mutex, OnceLock},
};

use tokio::sync::mpsc;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_RIGHTDOWN,
            MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
        },
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
            MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
            WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
        },
    },
};
use wm_common::{
    InvokeCommand, InvokeFocusCommand, InvokeMoveCommand, KeybindingConfig,
};

use super::PlatformEvent;

/// Global instance of `MouseHook`.
static MOUSE_HOOK: OnceLock<Arc<MouseHook>> = OnceLock::new();

/// Flag to indicate an injected event (to prevent infinite loops).
const INJECTED_EVENT_FLAG: u32 = 0xFF10;

#[derive(Debug, Default)]
struct MouseState {
    is_rmb_held: bool,
    has_rmb_scrolled: bool,
    is_lmb_held: bool,
}

#[derive(Debug)]
pub struct MouseHook {
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
    hook: Arc<Mutex<HHOOK>>,
    state: Arc<Mutex<MouseState>>,
}

impl MouseHook {
    pub fn new(event_tx: mpsc::UnboundedSender<PlatformEvent>) -> anyhow::Result<Arc<Self>> {
        let mouse_hook = Arc::new(Self {
            event_tx,
            hook: Arc::new(Mutex::new(HHOOK::default())),
            state: Arc::new(Mutex::new(MouseState::default())),
        });

        MOUSE_HOOK
            .set(mouse_hook.clone())
            .map_err(|_| anyhow::anyhow!("Mouse hook already running."))?;

        Ok(mouse_hook)
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let hook_handle = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
        }?;
        *self.hook.lock().unwrap() = hook_handle;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        let hook_handle = *self.hook.lock().unwrap();
        if !hook_handle.is_invalid() {
            unsafe { UnhookWindowsHookEx(hook_handle) }?;
        }
        Ok(())
    }

    fn send_command(&self, command: InvokeCommand) {
        let config = KeybindingConfig {
            commands: vec![command], 
            bindings: vec![],
        };
        let _ = self.event_tx.send(PlatformEvent::KeybindingTriggered(config));
    }

    fn replay_right_click(&self) {
        let inputs = [
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_RIGHTDOWN,
                        dwExtraInfo: INJECTED_EVENT_FLAG as usize,
                        ..Default::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_RIGHTUP,
                        dwExtraInfo: INJECTED_EVENT_FLAG as usize,
                        ..Default::default()
                    },
                },
            },
        ];
        unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32); }
    }
}

extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let hook_struct = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };
    
    if hook_struct.dwExtraInfo == INJECTED_EVENT_FLAG as usize {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    if let Some(mouse_hook) = MOUSE_HOOK.get() {
        let mut state = mouse_hook.state.lock().unwrap();
        let msg = wparam.0 as u32;

        match msg {
            WM_RBUTTONDOWN => {
                state.is_rmb_held = true;
                state.has_rmb_scrolled = false;
                return LRESULT(1);
            }
            WM_RBUTTONUP => {
                state.is_rmb_held = false;
                if state.has_rmb_scrolled {
                    return LRESULT(1);
                } else {
                    drop(state);
                    mouse_hook.replay_right_click();
                    return LRESULT(1);
                }
            }
            WM_LBUTTONDOWN => {
                state.is_lmb_held = true;
            }
            WM_LBUTTONUP => {
                state.is_lmb_held = false;
            }
            WM_MOUSEWHEEL => {
                let delta = (hook_struct.mouseData >> 16) as i16;
                let scroll_up = delta > 0;

                if state.is_rmb_held {
                    state.has_rmb_scrolled = true;
                    // RMB + Scroll: Switch Workspace
                    let cmd = InvokeCommand::Focus(InvokeFocusCommand {
                        direction: None, container_id: None, workspace_in_direction: None, workspace: None, monitor: None,
                        next_active_workspace: false, prev_active_workspace: false, next_workspace: false, prev_workspace: false,
                        next_active_workspace_on_monitor: false, prev_active_workspace_on_monitor: false, recent_workspace: false,
                        // REVERSED LOGIC:
                        next_workspace_on_monitor: scroll_up,   // Scroll Up -> Next
                        prev_workspace_on_monitor: !scroll_up,  // Scroll Down -> Prev
                    });
                    
                    mouse_hook.send_command(cmd); 
                    return LRESULT(1);
                }

                if state.is_lmb_held {
                    // LMB + Scroll: Move Window AND Follow
                    
                    // 1. Move the Window
                    let move_cmd = InvokeCommand::Move(InvokeMoveCommand {
                        direction: None, workspace_in_direction: None, workspace: None,
                        next_active_workspace: false, prev_active_workspace: false, next_workspace: false, prev_workspace: false,
                        next_active_workspace_on_monitor: false, prev_active_workspace_on_monitor: false, recent_workspace: false,
                        // REVERSED LOGIC:
                        next_workspace_on_monitor: scroll_up,
                        prev_workspace_on_monitor: !scroll_up,
                    });
                    mouse_hook.send_command(move_cmd);

                    // 2. Focus the Workspace (Move and Follow behavior)
                    let focus_cmd = InvokeCommand::Focus(InvokeFocusCommand {
                        direction: None, container_id: None, workspace_in_direction: None, workspace: None, monitor: None,
                        next_active_workspace: false, prev_active_workspace: false, next_workspace: false, prev_workspace: false,
                        next_active_workspace_on_monitor: false, prev_active_workspace_on_monitor: false, recent_workspace: false,
                        // REVERSED LOGIC:
                        next_workspace_on_monitor: scroll_up,
                        prev_workspace_on_monitor: !scroll_up,
                    });
                    mouse_hook.send_command(focus_cmd);

                    return LRESULT(1);
                }
            }
            _ => {}
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}