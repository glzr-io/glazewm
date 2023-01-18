using GlazeWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class WindowsApiService
  {
    [Flags]
    public enum SWP : uint
    {
      SWP_NOSIZE = 0x0001,
      SWP_NOMOVE = 0x0002,
      SWP_NOZORDER = 0x0004,
      SWP_NOREDRAW = 0x0008,
      SWP_NOACTIVATE = 0x0010,
      SWP_FRAMECHANGED = 0x0020,
      SWP_SHOWWINDOW = 0x0040,
      SWP_HIDEWINDOW = 0x0080,
      SWP_NOCOPYBITS = 0x0100,
      SWP_NOOWNERZORDER = 0x0200,
      SWP_NOSENDCHANGING = 0x0400,
      SWP_DEFERERASE = 0x2000,
      SWP_ASYNCWINDOWPOS = 0x4000
    }

    /// <summary>
    /// Flags that can be passed as `hWndInsertAfter` to `SetWindowPos`.
    /// </summary>
    public enum ZOrderFlags
    {
      /// <summary>
      /// Places the window above all non-topmost windows (that is, behind all topmost
      /// windows). This flag has no effect if the window is already a non-topmost window.
      /// </summary>
      NoTopMost = -2,
      /// <summary>
      /// Places the window above all non-topmost windows. The window maintains its
      /// topmost position even when it is deactivated.
      /// </summary>
      TopMost = -1,
      /// <summary>
      /// Places the window at the top of the Z order.
      /// </summary>
      Top = 0,
      /// <summary>
      /// Places the window at the bottom of the Z order.
      /// </summary>
      Bottom = 1,
    }

    /// <summary>
    /// Window styles
    /// </summary>
    [Flags]
    public enum WS : uint
    {
      WS_OVERLAPPED = 0x00000000,
      WS_TILED = WS_OVERLAPPED,
      WS_TABSTOP = 0x00010000,
      WS_MAXIMIZEBOX = 0x00010000,
      WS_GROUP = 0x00020000,
      WS_MINIMIZEBOX = 0x00020000,
      WS_THICKFRAME = 0x00040000,
      WS_SIZEBOX = WS_THICKFRAME,
      WS_SYSMENU = 0x00080000,
      WS_HSCROLL = 0x00100000,
      WS_VSCROLL = 0x00200000,
      WS_DLGFRAME = 0x00400000,
      WS_BORDER = 0x00800000,
      WS_CAPTION = WS_BORDER | WS_DLGFRAME,
      WS_TILEDWINDOW = WS_OVERLAPPEDWINDOW,
      WS_OVERLAPPEDWINDOW = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
      WS_MAXIMIZE = 0x01000000,
      WS_CLIPCHILDREN = 0x02000000,
      WS_CLIPSIBLINGS = 0x04000000,
      WS_DISABLED = 0x08000000,
      WS_VISIBLE = 0x10000000,
      WS_MINIMIZE = 0x20000000,
      WS_ICONIC = WS_MINIMIZE,
      WS_CHILD = 0x40000000,
      WS_CHILDWINDOW = WS_CHILD,
      WS_POPUP = 0x80000000,
      WS_POPUPWINDOW = WS_POPUP | WS_BORDER | WS_SYSMENU
    }

    /// <summary>
    /// Extended window styles
    /// </summary>
    [Flags]
    public enum WS_EX : uint
    {
      WS_EX_LEFT = 0x0000,
      WS_EX_LTRREADING = 0x0000,
      WS_EX_RIGHTSCROLLBAR = 0x0000,
      WS_EX_DLGMODALFRAME = 0x0001,
      WS_EX_NOPARENTNOTIFY = 0x0004,
      WS_EX_TOPMOST = 0x0008,
      WS_EX_ACCEPTFILES = 0x0010,
      WS_EX_TRANSPARENT = 0x0020,
      WS_EX_MDICHILD = 0x0040,
      WS_EX_TOOLWINDOW = 0x0080,
      WS_EX_WINDOWEDGE = 0x0100,
      WS_EX_PALETTEWINDOW = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
      WS_EX_CLIENTEDGE = 0x0200,
      WS_EX_OVERLAPPEDWINDOW = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE,
      WS_EX_CONTEXTHELP = 0x0400,
      WS_EX_RIGHT = 0x1000,
      WS_EX_RTLREADING = 0x2000,
      WS_EX_LEFTSCROLLBAR = 0x4000,
      WS_EX_CONTROLPARENT = 0x10000,
      WS_EX_STATICEDGE = 0x20000,
      WS_EX_APPWINDOW = 0x40000,
      WS_EX_LAYERED = 0x00080000,
      WS_EX_NOINHERITLAYOUT = 0x00100000,
      WS_EX_LAYOUTRTL = 0x00400000,
      WS_EX_COMPOSITED = 0x02000000,
      WS_EX_NOACTIVATE = 0x08000000
    }

    [Flags]
    public enum DwmWindowAttribute : uint
    {
      DWMWA_NCRENDERING_ENABLED = 1,
      DWMWA_NCRENDERING_POLICY,
      DWMWA_TRANSITIONS_FORCEDISABLED,
      DWMWA_ALLOW_NCPAINT,
      DWMWA_CAPTION_BUTTON_BOUNDS,
      DWMWA_NONCLIENT_RTL_LAYOUT,
      DWMWA_FORCE_ICONIC_REPRESENTATION,
      DWMWA_FLIP3D_POLICY,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      DWMWA_HAS_ICONIC_BITMAP,
      DWMWA_DISALLOW_PEEK,
      DWMWA_EXCLUDED_FROM_PEEK,
      DWMWA_CLOAK,
      DWMWA_CLOAKED,
      DWMWA_FREEZE_REPRESENTATION,
      DWMWA_LAST
    }

    public const int GWL_STYLE = -16;
    public const int GWL_EXSTYLE = -20;

    public enum GW : uint
    {
      GW_OWNER = 4,
    }

    public delegate bool EnumWindowsDelegate(IntPtr hWnd, int lParam);

    [DllImport("user32.dll", EntryPoint = "EnumWindows", ExactSpelling = false, CharSet = CharSet.Auto, SetLastError = true)]
    public static extern bool EnumWindows(EnumWindowsDelegate enumCallback, IntPtr lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll", EntryPoint = "GetWindowLong")]
    private static extern IntPtr GetWindowLongPtr32(IntPtr hWnd, int index);

    [DllImport("user32.dll", EntryPoint = "GetWindowLongPtr")]
    private static extern IntPtr GetWindowLongPtr64(IntPtr hWnd, int index);

    public static IntPtr GetWindowLongPtr(IntPtr hWnd, int index)
    {
      return Environment.Is64BitProcess
        ? GetWindowLongPtr64(hWnd, index)
        : GetWindowLongPtr32(hWnd, index);
    }

    [DllImport("user32.dll", EntryPoint = "SetWindowLong")]
    private static extern IntPtr SetWindowLongPtr32(IntPtr hWnd, int index, IntPtr newLong);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")]
    private static extern IntPtr SetWindowLongPtr64(IntPtr hWnd, int index, IntPtr newLong);

    public static IntPtr SetWindowLongPtr(IntPtr hWnd, int index, IntPtr newLong)
    {
      return Environment.Is64BitProcess
        ? SetWindowLongPtr64(hWnd, index, newLong)
        : SetWindowLongPtr32(hWnd, index, newLong);
    }

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, SWP uFlags);

    [DllImport("user32.dll")]
    public static extern IntPtr BeginDeferWindowPos(int nNumWindows);

    [DllImport("user32.dll")]
    public static extern IntPtr DeferWindowPos(IntPtr hWinPosInfo, IntPtr hWnd,
         [Optional] IntPtr hWndInsertAfter, int x, int y, int cx, int cy, SWP uFlags);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool EndDeferWindowPos(IntPtr hWinPosInfo);

    [DllImport("user32.dll")]
    public static extern IntPtr GetDesktopWindow();

    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool MoveWindow(IntPtr hWnd, int X, int Y, int nWidth, int nHeight, bool bRepaint);

    [DllImport("user32.dll")]
    public static extern bool SetFocus(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern IntPtr WindowFromPoint(Point Point);

    [DllImport("user32.dll", ExactSpelling = true, CharSet = CharSet.Auto)]
    public static extern IntPtr GetParent(IntPtr hWnd);
    public static extern bool SetCursorPos(int X, int Y);

    /// <summary>
    /// Params that can be passed to `ShowWindow`. Only the subset of flags relevant to
    /// this application are included.
    /// </summary>
    public enum ShowWindowCommands : uint
    {
      MINIMIZE = 2,
      MAXIMIZE = 3,
      RESTORE = 9,
    }

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, ShowWindowCommands flags);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetWindowTextLength(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, [Out] StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, ref Rect rectangle);

    /// <summary>
    /// Contains information about the placement of a window on the screen.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct WindowPlacement
    {
      /// <summary>
      /// The length of the structure, in bytes. Before calling the GetWindowPlacement or SetWindowPlacement functions, set this member to sizeof(WINDOWPLACEMENT).
      /// </summary>
      public int Length;

      /// <summary>
      /// Specifies flags that control the position of the minimized window and the method by which the window is restored.
      /// </summary>
      public int Flags;

      /// <summary>
      /// The current show state of the window.
      /// </summary>
      public ShowWindowCommands ShowCommand;

      /// <summary>
      /// The coordinates of the window's upper-left corner when the window is minimized.
      /// </summary>
      public Point MinPosition;

      /// <summary>
      /// The coordinates of the window's upper-left corner when the window is maximized.
      /// </summary>
      public Point MaxPosition;

      /// <summary>
      /// The window's coordinates when the window is in the restored position.
      /// </summary>
      public Rect NormalPosition;
    }

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetWindowPlacement(IntPtr hWnd, ref WindowPlacement windowPlacement);

    [DllImport("user32.dll")]
    public static extern IntPtr GetWindow(IntPtr hWnd, GW uCmd);

    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr hwnd, DwmWindowAttribute dwAttribute, out bool pvAttribute, int cbAttribute);

    [DllImport("user32.dll")]
    public static extern IntPtr GetShellWindow();

    public delegate IntPtr HookProc(int code, IntPtr wParam, IntPtr lParam);

    public enum HookType
    {
      WH_KEYBOARD_LL = 13,
      WH_MOUSE_LL = 14
    }

    /// <summary>
    /// Contains information about a low-level keyboard input event.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct LowLevelKeyboardInputEvent
    {
      /// <summary>
      /// A virtual-key code. The code must be a value in the range 1 to 254.
      /// </summary>
      public int VirtualCode;

      /// <summary>
      /// The `VirtualCode` converted to `Keys` for better usability.
      /// </summary>
      public Keys Key => (Keys)VirtualCode;

      /// <summary>
      /// A hardware scan code for the key.
      /// </summary>
      public int HardwareScanCode;

      /// <summary>
      /// The extended-key flag, event-injected Flags, context code, and transition-state flag. This member is specified as follows. An application can use the following values to test the keystroke Flags. Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected. If it was, then testing LLKHF_LOWER_IL_INJECTED (bit 1) will tell you whether or not the event was injected from a process running at lower integrity level.
      /// </summary>
      public int Flags;

      /// <summary>
      /// The time stamp for this message, equivalent to what GetMessageTime would return for this message.
      /// </summary>
      public int TimeStamp;

      /// <summary>
      /// Additional information associated with the message.
      /// </summary>
      public IntPtr AdditionalInformation;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LowLevelMouseInputEvent
    {
      /// <summary>
      /// (X,Y) location of mouse with 0,0 being the top-left corner of the main monitor 
      /// </summary>
      public Point pt;

      /// <summary>
      /// ?? What does this do ?? (TODO)
      /// </summary>
      public int mouseData;

      /// <summary>
      /// The extended-key flag, event-injected Flags, context code, and transition-state flag. This member is specified as follows. An application can use the following values to test the keystroke Flags. Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected. If it was, then testing LLKHF_LOWER_IL_INJECTED (bit 1) will tell you whether or not the event was injected from a process running at lower integrity level.
      /// </summary>
      public int Flags;

      /// <summary>
      /// The time stamp for this message, equivalent to what GetMessageTime would return for this message.
      /// </summary>
      public int TimeStamp;

      /// <summary>
      /// Additional information associated with the message.
      /// </summary>
      public IntPtr dwExtraInfo;
    }

    [DllImport("user32.dll")]
    public static extern IntPtr SetWindowsHookEx(HookType hookType, [MarshalAs(UnmanagedType.FunctionPtr)] HookProc lpfn, IntPtr hMod, int dwThreadId);

    [DllImport("user32.dll")]
    public static extern IntPtr CallNextHookEx([Optional] IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern int GetKeyboardState(byte[] pbKeyState);

    [DllImport("user32.dll")]
    public static extern short GetKeyState(Keys nVirtKey);

    [DllImport("user32.dll", EntryPoint = "keybd_event")]
    public static extern void KeybdEvent(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);

    [DllImport("User32.dll")]
    public static extern short GetAsyncKeyState(Keys key);

    public delegate void WindowEventProc(IntPtr hWinEventHook, EventConstant eventType, IntPtr hwnd, ObjectIdentifier idObject, int idChild, uint dwEventThread, uint dwmsEventTime);

    [DllImport("user32.dll")]
    public static extern IntPtr SetWinEventHook(EventConstant eventMin, EventConstant eventMax, IntPtr hmodWinEventProc, WindowEventProc lpfnWinEventProc, uint idProcess, uint idThread, uint dwFlags);

    /// <summary>
    /// Message types that can be passed to `SendMessage`. Only the subset of types relevant to
    /// this application are included.
    /// </summary>
    public enum SendMessageType : uint
    {
      WM_CLOSE = 0x0010,
    }

    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr SendMessage(IntPtr hWnd, SendMessageType Msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool IsProcessDPIAware();

    public enum DpiAwarenessContext
    {
      Context_UnawareGdiScaled = -5,
      Context_PerMonitorAwareV2 = -4,
      Context_PerMonitorAware = -3,
      Context_SystemAware = -2,
      Context_Unaware = -1,
      Context_Undefined = 0
    }

    [DllImport("user32.dll", SetLastError = true)]
    public static extern int SetProcessDpiAwarenessContext(DpiAwarenessContext value);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetDpiForWindow(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectEx(ref Rect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WS_EX dwExStyle);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRect(ref Rect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectExForDpi(ref Rect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WS_EX dwExStyle, uint dpi);

    public enum DpiType
    {
      Effective = 0,
      Angular = 1,
      Raw = 2,
    }

    [DllImport("Shcore.dll")]
    public static extern IntPtr GetDpiForMonitor(IntPtr hmonitor, DpiType dpiType, out uint dpiX, out uint dpiY);

    public enum MonitorFromPointFlags : uint
    {
      MONITOR_DEFAULTTONEAREST = 2,
    }

    [DllImport("User32.dll")]
    public static extern IntPtr MonitorFromPoint(Point pt, MonitorFromPointFlags dwFlags);

    [DllImport("kernel32.dll")]
    public static extern bool GetSystemPowerStatus(out SYSTEM_POWER_STATUS lpSystemPowerStatus);

    public struct SYSTEM_POWER_STATUS
    {
      public byte ACLineStatus;
      public byte BatteryFlag;
      public byte BatteryLifePercent;
      public byte SystemStatusFlag;
      public uint BatteryLifeTime;
      public uint BatteryFullLifeTime;
    }
  }
}
