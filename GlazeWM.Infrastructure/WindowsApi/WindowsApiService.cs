using GlazeWM.Infrastructure.WindowsApi.Enums;
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class WindowsApiService
  {
    public enum SWP : uint
    {
      SWP_SHOWWINDOW = 0x0040,
      SWP_HIDEWINDOW = 0x0080,
      SWP_NOZORDER = 0x0004,
      SWP_NOREDRAW = 0x0008,
      SWP_NOACTIVATE = 0x0010,
      SWP_NOMOVE = 0x0002,
      SWP_NOSIZE = 0x0001,
      SWP_FRAMECHANGED = 0x0020,
      SWP_NOCOPYBITS = 0x0100,
      SWP_NOOWNERZORDER = 0x0200,
      SWP_DEFERERASE = 0x2000,
      SWP_NOSENDCHANGING = 0x0400,
      SWP_ASYNCWINDOWPOS = 0x4000
    }

    /// <summary>
    /// Window styles
    /// </summary>
    [Flags]
    public enum WS : int
    {
      WS_OVERLAPPED = 0,
      WS_POPUP = unchecked((int)0x80000000),
      WS_CHILD = 0x40000000,
      WS_MINIMIZE = 0x20000000,
      WS_VISIBLE = 0x10000000,
      WS_DISABLED = 0x8000000,
      WS_CLIPSIBLINGS = 0x4000000,
      WS_CLIPCHILDREN = 0x2000000,
      WS_MAXIMIZE = 0x1000000,
      WS_CAPTION = WS_BORDER | WS_DLGFRAME,
      WS_BORDER = 0x800000,
      WS_DLGFRAME = 0x400000,
      WS_VSCROLL = 0x200000,
      WS_HSCROLL = 0x100000,
      WS_SYSMENU = 0x80000,
      WS_THICKFRAME = 0x40000,
      WS_MINIMIZEBOX = 0x20000,
      WS_MAXIMIZEBOX = 0x10000,
      WS_TILED = WS_OVERLAPPED,
      WS_ICONIC = WS_MINIMIZE,
      WS_SIZEBOX = WS_THICKFRAME,
      WS_OVERLAPPEDWINDOW = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX
    }

    /// <summary>
    /// Extended window styles
    /// </summary>
    [Flags]
    public enum WS_EX : uint
    {
      WS_EX_DLGMODALFRAME = 0x0001,
      WS_EX_NOPARENTNOTIFY = 0x0004,
      WS_EX_TOPMOST = 0x0008,
      WS_EX_ACCEPTFILES = 0x0010,
      WS_EX_TRANSPARENT = 0x0020,
      WS_EX_MDICHILD = 0x0040,
      WS_EX_TOOLWINDOW = 0x0080,
      WS_EX_WINDOWEDGE = 0x0100,
      WS_EX_CLIENTEDGE = 0x0200,
      WS_EX_CONTEXTHELP = 0x0400,
      WS_EX_RIGHT = 0x1000,
      WS_EX_LEFT = 0x0000,
      WS_EX_RTLREADING = 0x2000,
      WS_EX_LTRREADING = 0x0000,
      WS_EX_LEFTSCROLLBAR = 0x4000,
      WS_EX_RIGHTSCROLLBAR = 0x0000,
      WS_EX_CONTROLPARENT = 0x10000,
      WS_EX_STATICEDGE = 0x20000,
      WS_EX_APPWINDOW = 0x40000,
      WS_EX_OVERLAPPEDWINDOW = (WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE),
      WS_EX_PALETTEWINDOW = (WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST),
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

    public static int GWL_STYLE = -16;
    public static int GWL_EXSTYLE = -20;

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
      => Environment.Is64BitProcess ? GetWindowLongPtr64(hWnd, index) : GetWindowLongPtr32(hWnd, index);

    [DllImport("user32.dll", EntryPoint = "SetWindowLong")]
    private static extern IntPtr SetWindowLongPtr32(IntPtr hWnd, int index, IntPtr newLong);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")]
    private static extern IntPtr SetWindowLongPtr64(IntPtr hWnd, int index, IntPtr newLong);

    public static IntPtr SetWindowLongPtr(IntPtr hWnd, int index, IntPtr newLong)
      => Environment.Is64BitProcess ? SetWindowLongPtr64(hWnd, index, newLong) : SetWindowLongPtr32(hWnd, index, newLong);

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

    /// <summary>
    /// Params that can be passed to `ShowWindow`. Only the subset of flags relevant to
    /// this application are included.
    /// </summary>
    public enum ShowWindowFlags : uint
    {
      RESTORE = 9,
    }

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, ShowWindowFlags flags);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetWindowTextLength(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetWindowText(IntPtr hWnd, [Out] StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, ref WindowRect rectangle);

    [DllImport("user32.dll")]
    public static extern IntPtr GetWindow(IntPtr hWnd, GW uCmd);

    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr hwnd, DwmWindowAttribute dwAttribute, out bool pvAttribute, int cbAttribute);

    [DllImport("user32.dll")]
    public static extern IntPtr GetShellWindow();

    public delegate IntPtr HookProc(int code, IntPtr wParam, IntPtr lParam);

    public enum HookType : int
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
      public Keys Key { get { return (Keys)VirtualCode; } }

      /// <summary>
      /// A hardware scan code for the key.
      /// </summary>
      public int HardwareScanCode;

      /// <summary>
      /// The extended-key flag, event-injected Flags, context code, and transition-state flag. This member is specified as follows. An application can use the following values to test the keystroke Flags. Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected. If it was, then testing LLKHF_LOWER_IL_INJECTED (bit 1) will tell you whether or not the event was injected from a process running at lower integrity level.
      /// </summary>
      public int Flags;

      /// <summary>
      /// The time stamp stamp for this message, equivalent to what GetMessageTime would return for this message.
      /// </summary>
      public int TimeStamp;

      /// <summary>
      /// Additional information associated with the message.
      /// </summary>
      public IntPtr AdditionalInformation;
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
      Context_Undefined = 0,
      Context_Unaware = -1,
      Context_SystemAware = -2,
      Context_PerMonitorAware = -3,
      Context_PerMonitorAwareV2 = -4,
      Context_UnawareGdiScaled = -5
    }

    [DllImport("user32.dll", SetLastError = true)]
    public static extern int SetProcessDpiAwarenessContext(DpiAwarenessContext value);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetDpiForWindow(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectEx(ref WindowRect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WS_EX dwExStyle);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRect(ref WindowRect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectExForDpi(ref WindowRect lpRect, WS dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WS_EX dwExStyle, uint dpi);

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
    public static extern IntPtr MonitorFromPoint(System.Drawing.Point pt, MonitorFromPointFlags dwFlags);
  }
}
