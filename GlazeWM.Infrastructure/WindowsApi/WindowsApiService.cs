using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;
using GlazeWM.Infrastructure.WindowsApi.Enums;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class WindowsApiService
  {
    [Flags]
    public enum SetWindowPosFlags : uint
    {
      NoSize = 0x0001,
      NoMove = 0x0002,
      NoZOrder = 0x0004,
      NoRedraw = 0x0008,
      NoActivate = 0x0010,
      FrameChanged = 0x0020,
      ShowWindow = 0x0040,
      HideWindow = 0x0080,
      NoCopyBits = 0x0100,
      NoOwnerZOrder = 0x0200,
      NoSendChanging = 0x0400,
      DeferErase = 0x2000,
      AsyncWindowPos = 0x4000
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
    public enum WindowStyles : uint
    {
      Overlapped = 0x00000000,
      Tiled = Overlapped,
      TabStop = 0x00010000,
      MaximizeBox = 0x00010000,
      Group = 0x00020000,
      MinimizeBox = 0x00020000,
      ThickFrame = 0x00040000,
      SizeBox = ThickFrame,
      SysMenu = 0x00080000,
      HScroll = 0x00100000,
      VScroll = 0x00200000,
      DlgFrame = 0x00400000,
      Border = 0x00800000,
      Capion = Border | DlgFrame,
      TiledWindow = OverlappedWindow,
      OverlappedWindow = Overlapped | Capion | SysMenu | ThickFrame | MinimizeBox | MaximizeBox,
      Maximize = 0x01000000,
      ClipChildren = 0x02000000,
      ClipSiblings = 0x04000000,
      Disabled = 0x08000000,
      Visible = 0x10000000,
      Minimize = 0x20000000,
      Iconic = Minimize,
      Child = 0x40000000,
      ChildWindow = Child,
      Popup = 0x80000000,
      PopupWindow = Popup | Border | SysMenu
    }

    /// <summary>
    /// Extended window styles
    /// </summary>
    [Flags]
    public enum WindowStylesEx : uint
    {
      Left = 0x0000,
      LtrReading = 0x0000,
      RightScrollbar = 0x0000,
      DlgModalFrame = 0x0001,
      NoParentNotify = 0x0004,
      TopMost = 0x0008,
      AcceptFiles = 0x0010,
      Transparent = 0x0020,
      MdiChild = 0x0040,
      ToolWindow = 0x0080,
      WindowEdge = 0x0100,
      PaletteWindow = WindowEdge | ToolWindow | TopMost,
      ClientEdge = 0x0200,
      OverlappedWindow = WindowEdge | ClientEdge,
      ContextHelp = 0x0400,
      Right = 0x1000,
      RtlReading = 0x2000,
      LeftScrollbar = 0x4000,
      ControlParent = 0x10000,
      StaticEdge = 0x20000,
      AppWindow = 0x40000,
      Layered = 0x00080000,
      NoInheritLayout = 0x00100000,
      LayoutRtl = 0x00400000,
      Composited = 0x02000000,
      NoActivate = 0x08000000
    }

    [Flags]
    public enum DwmWindowAttribute : uint
    {
      NcRenderingEnabled = 1,
      NcRenderingPolicy,
      TransitionsForceDisabled,
      AllowNcPaint,
      CaptionButtonBounds,
      NonClientRtlLayout,
      ForceIconicRepresentation,
      Flip3DPolicy,
      ExtendedFrameBounds,
      HasIconicBitmap,
      DisallowPeek,
      ExcludedFromPeek,
      Cloak,
      Cloaked,
      FreezeRepresentation,
      Last
    }

    public const int GWLSTYLE = -16;
    public const int GWLEXSTYLE = -20;

    public enum GW : uint
    {
      Owner = 4,
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
    public static extern bool SetWindowPos(
      IntPtr hWnd,
      IntPtr hWndInsertAfter,
      int x,
      int y,
      int cx,
      int cy,
      SetWindowPosFlags uFlags);

    [DllImport("user32.dll")]
    public static extern IntPtr BeginDeferWindowPos(int nNumWindows);

    [DllImport("user32.dll")]
    public static extern IntPtr DeferWindowPos(
      IntPtr hWinPosInfo,
      IntPtr hWnd,
      [Optional] IntPtr hWndInsertAfter,
      int x,
      int y,
      int cx,
      int cy,
      SetWindowPosFlags uFlags);

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
    public static extern bool MoveWindow(IntPtr hWnd, int x, int y, int nWidth, int nHeight, bool bRepaint);

    [DllImport("user32.dll")]
    public static extern bool SetFocus(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern IntPtr WindowFromPoint(Point Point);

    [DllImport("user32.dll")]
    public static extern IntPtr GetParent(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);

    /// <summary>
    /// Params that can be passed to `ShowWindow`. Only the subset of flags relevant to
    /// this application are included.
    /// </summary>
    public enum ShowWindowFlags : uint
    {
      Minimize = 2,
      Maximize = 3,
      Restore = 9,
    }

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, ShowWindowFlags flags);

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
      public ShowWindowFlags ShowCommand;

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
      KeyboardLowLevel = 13,
      MouseLowLevel = 14
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
      /// Mouse data.
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
    public static extern bool UnhookWindowsHookEx(IntPtr hhk);

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
      Close = 0x0010,
    }

    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr SendMessage(IntPtr hWnd, SendMessageType msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool IsProcessDPIAware();

    public enum DpiAwarenessContext
    {
      UnawareGdiScaled = -5,
      PerMonitorAwareV2 = -4,
      PerMonitorAware = -3,
      SystemAware = -2,
      Unaware = -1,
      Undefined = 0
    }

    [DllImport("user32.dll", SetLastError = true)]
    public static extern int SetProcessDpiAwarenessContext(DpiAwarenessContext value);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetDpiForWindow(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectEx(ref Rect lpRect, WindowStyles dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WindowStylesEx dwExStyle);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRect(ref Rect lpRect, WindowStyles dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool AdjustWindowRectExForDpi(ref Rect lpRect, WindowStyles dwStyle, [MarshalAs(UnmanagedType.Bool)] bool bMenu, WindowStylesEx dwExStyle, uint dpi);

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
      DefaultToNearest = 2,
    }

    [DllImport("User32.dll")]
    public static extern IntPtr MonitorFromPoint(Point pt, MonitorFromPointFlags dwFlags);

    [DllImport("kernel32.dll")]
    public static extern bool GetSystemPowerStatus(out SystemPowerStatus lpSystemPowerStatus);

    [StructLayout(LayoutKind.Sequential)]
    public struct SystemPowerStatus
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
