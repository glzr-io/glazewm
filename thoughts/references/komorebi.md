## Komorebi Windows API usage map

Concise, per-file reference of the Win32/COM/DWM/Direct2D APIs used by `komorebi` and `komorebi-bar`. Use this as a guide for calling User32, GDI, DWM, Shell, Accessibility, Direct2D, and related APIs from Rust via the `windows` crate.

### Crate-to-API mapping

- **windows**: Win32 (User32/WindowsAndMessaging, GDI, DWM, Shell, HiDPI, Threading, Power, RemoteDesktop, Accessibility), Direct2D/DXGI, Globalization, Foundation
- **windows-core / windows_core**: `HSTRING`, `PCWSTR`, `PWSTR`, `BOOL`, COM helpers
- **win32-display-data**: EDID/Display config → `HMONITOR`, device path/name, work area

## Files and APIs

### komorebi/src/windows_api.rs

- **User32 WindowsAndMessaging**: `AllowSetForegroundWindow`, `BringWindowToTop`, `CreateWindowExW`, `EnumWindows`, `GetCursorPos`, `GetDesktopWindow`, `GetForegroundWindow`, `GetLayeredWindowAttributes`, `GetTopWindow`, `GetWindow`, `GetWindowLongPtrW`, `GetWindowRect`, `GetWindowTextW`, `GetWindowThreadProcessId`, `IsIconic`, `IsWindow`, `IsWindowVisible`, `IsZoomed`, `MoveWindow`, `PostMessageW`, `RealGetWindowClassW`, `RegisterClassW`, `RegisterDeviceNotificationW`, `SendMessageW`, `SetCursorPos`, `SetForegroundWindow`, `SetLayeredWindowAttributes`, `SetWindowLongPtrW`, `SetWindowPos`, `ShowWindow`/`ShowWindowAsync`, `SystemParametersInfoW`, `WindowFromPoint`
- **GDI**: `CreateSolidBrush`, `EnumDisplayMonitors`, `GetMonitorInfoW`, `InvalidateRect`, `MonitorFromPoint`, `MonitorFromWindow`, `Rectangle`, `RoundRect`, `UpdateWindow`
- **DWM**: `DwmGetWindowAttribute` (`DWMWA_EXTENDED_FRAME_BOUNDS`, `DWMWA_CLOAKED`), `DwmSetWindowAttribute` (`DWMWA_WINDOW_CORNER_PREFERENCE`, `DWMWA_BORDER_COLOR`)
- **HiDPI**: `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`, `GetDpiForMonitor(MDT_EFFECTIVE_DPI)`
- **Input**: `SendInput`, `GetKeyState`, `VK_MENU`, `VK_LBUTTON`
- **Threading/Processes**: `GetCurrentProcessId`, `OpenProcess`, `QueryFullProcessImageNameW`, `PROCESS_QUERY_INFORMATION`
- **Shell (COM)**: `IDesktopWallpaper::SetWallpaper/GetWallpaper`, `DesktopWallpaper` via `CoCreateInstance`
- **COM/Module**: `GetModuleHandleW`, `CoCreateInstance`
- **Power/Session**: `RegisterPowerSettingNotification`, `WTSRegisterSessionNotification`, `ProcessIdToSessionId`
- Purpose: Central Win32 wrapper for monitor enumeration, window discovery, sizing/positioning (incl. async `SetWindowPos` flags), z-order/focus, transparency, DPI, wallpaper, and system parameters.

### komorebi/src/windows_callbacks.rs

- **User32**: `GetWindowLongW(GWL_STYLE/GWL_EXSTYLE)`, `SendNotifyMessageW`
- Purpose: Enum callbacks for window enumeration and WinEvent hook; filters child/tool/noactivate windows; forwards move/destroy to border manager.

### komorebi/src/winevent_listener.rs

- **Accessibility**: `SetWinEventHook(EVENT_MIN..EVENT_MAX, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)`
- **Message loop**: `GetMessageW`, `TranslateMessage`, `DispatchMessageW`
- Purpose: Global WinEvent pump thread delivering `WindowManagerEvent`s.

### komorebi/src/winevent.rs

- **User32**: `EVENT_*` constants mapped to `WinEvent` enum
- Purpose: Typed mapping from WinEvent numeric IDs to Rust enum.

### komorebi/src/window.rs

- **DWM**: `DwmFlush`
- Purpose: High-level window ops built on `WindowsApi` (move/resize/focus/maximize/minimize, styles, transparency, accent); uses private COM cloak via `com::SetCloak`.

### komorebi/src/styles.rs

- **User32 style constants**: `WS_*`, `WS_EX_*`
- Purpose: Bitflags for standard and extended window styles.

### komorebi/src/stackbar_manager/stackbar.rs

- **User32**: `CreateWindowExW`, `WNDCLASSW`, `DefWindowProcW`, `GetMessageW`, `TranslateMessage`, `DispatchMessageW`, `LoadCursorW`, `SetCursor`, `PostQuitMessage`, `SetLayeredWindowAttributes(LWA_COLORKEY)`, `WM_*`
- **GDI**: `GetDC/ReleaseDC`, `CreatePen`, `CreateSolidBrush`, `CreateFontIndirectW`, `SelectObject`, `SetBkColor`, `SetTextColor`, `DrawTextW`, `Rectangle`, `RoundRect`, `DeleteObject`, `MulDiv`
- Purpose: Owner-drawn layered popup window for per-container tabs.

### komorebi/src/border_manager/border.rs

- **Direct2D**: `D2D1CreateFactory`, `ID2D1HwndRenderTarget`, `CreateHwndRenderTarget`, `CreateSolidColorBrush`, `DrawRoundedRectangle`/`DrawRectangle`, `EndDraw`
- **DXGI**: `DXGI_FORMAT_UNKNOWN`
- **DWM**: `DwmEnableBlurBehindWindow`
- **GDI**: `CreateRectRgn`, `InvalidateRect`, `ValidateRect`
- **User32**: `WNDCLASSW`, `DefWindowProcW`, `GetMessageW`, `TranslateMessage`, `DispatchMessageW`, `LoadCursorW`, `SetCursor`, `GetWindowLongPtrW/GWLP_USERDATA`, `SetWindowLongPtrW`, `PostQuitMessage`, `WM_*` incl. `EVENT_OBJECT_LOCATIONCHANGE/DESTROY`
- Purpose: Per-window overlay border windows, rendered via Direct2D; keeps size/position in sync with tracked HWND.

### komorebi/src/border_manager/mod.rs

- Types: `ID2D1HwndRenderTarget`, `HWND`
- Purpose: Border orchestration; calls `WindowsApi` to show/hide/raise/lower and enumerates border windows.

### komorebi/src/monitor_reconciliator/hidden.rs

- **User32**: hidden window class/loop (`WNDCLASSW`, `DefWindowProcW`, `GetMessageW`, `TranslateMessage`, `DispatchMessageW`), `WM_POWERBROADCAST`, `WM_WTSSESSION_CHANGE`, `WM_DISPLAYCHANGE`, `WM_SETTINGCHANGE`, `WM_DEVICECHANGE`
- **System GUIDs**: `GUID_LIDSWITCH_STATE_CHANGE`
- **Devices/Display GUIDs**: `GUID_DEVINTERFACE_MONITOR`, `GUID_DEVINTERFACE_DISPLAY_ADAPTER`, `GUID_DEVINTERFACE_VIDEO_OUTPUT_ARRIVAL`
- **Structures**: `DEV_BROADCAST_DEVICEINTERFACE_W`, `POWERBROADCAST_SETTING`
- Purpose: Message-only hidden window to subscribe to power/session/monitor device changes; registers for WTS session, power settings, and device notifications via `WindowsApi`.

### komorebi/src/monitor_reconciliator/mod.rs

- Purpose: Uses `WindowsApi::monitor`, `WindowsApi::load_monitor_information` and `win32-display-data` to reconcile attach/detach, scaling, and work areas (no direct Win32 calls outside tests).

### komorebi/src/set_window_position.rs

- **User32**: `SWP_*` flags → `SetWindowPosition` bitflags
- Purpose: Type-safe wrapper around `SetWindowPos` flags.

### komorebi/src/core/rect.rs

- **Foundation**: `RECT`
- Purpose: Conversions and helpers between internal `Rect` and Win32 `RECT`.

### komorebi/src/com/mod.rs and komorebi/src/com/interfaces.rs

- **COM init**: `CoInitializeEx(COINIT_MULTITHREADED)`, `CoUninitialize`
- **COM activation**: `CoCreateInstance(CLSID_ImmersiveShell)` → `IServiceProvider`
- **Custom interfaces**: `IServiceProvider::QueryService`, `IApplicationViewCollection::get_view_for_hwnd`, `IApplicationView::set_cloak`
- Purpose: Private Immersive Shell COM to cloak/uncloak windows backing `SetCloak(HWND, cloak_type, flags)`.

### komorebi-bar/src/main.rs

- **HiDPI**: `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`
- **Threading/User32**: `GetCurrentProcessId`, `GetCurrentThreadId`, `EnumThreadWindows`, `GetWindowThreadProcessId`
- Purpose: Discover own HWND for z-order/positioning and set DPI awareness before creating egui window.

### komorebi-bar/src/widgets/keyboard.rs

- **User32**: `GetForegroundWindow`, `GetWindowThreadProcessId`, `GetKeyboardLayout`
- **Globalization**: `LCIDToLocaleName`, `LOCALE_NAME_MAX_LENGTH`
- Purpose: Resolve active keyboard layout name for the foreground window’s thread.

## Notes

- Many higher-level operations route through `WindowsApi` to centralize error handling and pointer conversions (`HWND`, `RECT`, `PCWSTR`).
- Private COM (Immersive Shell) use is undocumented and may change; isolated in `com/*` and invoked via `SetCloak`.
- For monitor enumeration and wallpaper control, the project mixes Win32 (`EnumDisplayMonitors`, `GetMonitorInfoW`) with `win32-display-data` and Shell `IDesktopWallpaper`.
