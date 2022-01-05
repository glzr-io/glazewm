using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public class WindowService
  {
    /// <summary>
    /// Window handles to appbars (application desktop toolbars). Positioning changes of appbars
    /// can affect the working area of the parent monitor and requires windows to be redrawn.
    /// </summary>
    public List<IntPtr> AppBarHandles { get; set; } = new List<IntPtr>();

    private ContainerService _containerService;
    private MonitorService _monitorService;

    public WindowService(ContainerService containerService, MonitorService monitorService)
    {
      _containerService = containerService;
      _monitorService = monitorService;
    }

    /// <summary>
    /// Get all windows by traversing down container tree.
    /// </summary>
    public IEnumerable<Window> GetWindows()
    {
      return _containerService.ContainerTree.Descendants.OfType<Window>();
    }

    /// <summary>
    /// Get the id of the process that created the window.
    /// </summary>
    public Process GetProcessOfHandle(IntPtr handle)
    {
      uint processId;
      GetWindowThreadProcessId(handle, out processId);
      return Process.GetProcesses().FirstOrDefault(process => process.Id == (int)processId);
    }

    /// <summary>
    /// Get the class name of the specified window.
    /// </summary>
    public string GetClassNameOfHandle(IntPtr handle)
    {
      // Class name is limited to 256 characters, so it's fine to use a fixed size buffer.
      var buffer = new StringBuilder(256);
      GetClassName(handle, buffer, buffer.Capacity);
      return buffer.ToString();
    }

    /// <summary>
    /// Get dimensions of the bounding rectangle of the specified window.
    /// </summary>
    public WindowRect GetLocationOfHandle(IntPtr handle)
    {
      var rect = new WindowRect();
      GetWindowRect(handle, ref rect);
      return rect;
    }

    /// <summary>
    /// Get info about the placement of the specified window.
    /// </summary>
    public WindowPlacement GetPlacementOfHandle(IntPtr handle)
    {
      var windowPlacement = new WindowPlacement();
      GetWindowPlacement(handle, ref windowPlacement);
      return windowPlacement;
    }

    /// <summary>
    /// Get title bar text of the specified window.
    /// </summary>
    public string GetTitleOfHandle(IntPtr handle)
    {
      var titleLength = GetWindowTextLength(handle);

      if (titleLength == 0)
        return String.Empty;

      var buffer = new StringBuilder(titleLength + 1);
      GetWindowText(handle, buffer, buffer.Capacity);
      return buffer.ToString();
    }

    public List<IntPtr> GetAllWindowHandles()
    {
      var windowHandles = new List<IntPtr>();

      EnumWindows((IntPtr hwnd, int lParam) =>
      {
        windowHandles.Add(hwnd);
        return true;
      }, IntPtr.Zero);

      return windowHandles;
    }

    public WS_EX GetWindowStylesEx(IntPtr handle)
    {
      return unchecked((WS_EX)GetWindowLongPtr(handle, (int)(GWL_EXSTYLE)).ToInt64());
    }

    public WS GetWindowStyles(IntPtr handle)
    {
      return unchecked((WS)GetWindowLongPtr(handle, (int)(GWL_STYLE)).ToInt64());
    }

    public bool HandleHasWindowStyle(IntPtr handle, WS style)
    {
      return (GetWindowStyles(handle) & style) != 0;
    }

    public bool HandleHasWindowExStyle(IntPtr handle, WS_EX style)
    {
      return (GetWindowStylesEx(handle) & style) != 0;
    }

    /// <summary>
    /// Whether the given handle is cloaked. For some UWP apps, `WS_VISIBLE` will be true even if
    /// the window isn't actually visible. The `DWMWA_CLOAKED` attribute is used to check whether
    /// these apps are visible.
    /// </summary>
    public bool IsHandleCloaked(IntPtr handle)
    {
      bool isCloaked;
      DwmGetWindowAttribute(handle, DwmWindowAttribute.DWMWA_CLOAKED, out isCloaked, Marshal.SizeOf(typeof(bool)));
      return isCloaked;
    }

    /// <summary>
    /// Whether the given handle is actually visible.
    /// </summary>
    public bool IsHandleVisible(IntPtr handle)
    {
      return IsWindowVisible(handle) && !IsHandleCloaked(handle);
    }

    public bool IsHandleManageable(IntPtr handle)
    {
      // Ignore windows that are hidden.
      if (!IsHandleVisible(handle))
        return false;

      // Ensure window is top-level (ie. not a child window). Ignore windows that cannot be focused
      // or if they're unavailable in task switcher (alt+tab menu).
      var isApplicationWindow = !HandleHasWindowStyle(handle, WS.WS_CHILD)
        && !HandleHasWindowExStyle(handle, WS_EX.WS_EX_NOACTIVATE | WS_EX.WS_EX_TOOLWINDOW);

      if (!isApplicationWindow)
        return false;

      // Get whether the window belongs to the current process.
      var isCurrentProcess = GetProcessOfHandle(handle).Id == Process.GetCurrentProcess().Id;

      if (isCurrentProcess)
        return false;

      return true;
    }

    /// <summary>
    /// Whether the given handle is an appbar window (application desktop toolbar).
    /// </summary>
    public bool IsHandleAppBar(IntPtr handle)
    {
      // Appbar window has to be visible.
      if (!IsHandleVisible(handle))
        return false;

      var monitor = _monitorService.GetMonitorFromHandleLocation(handle);
      var location = GetLocationOfHandle(handle);

      var isFullWidth = location.Width == monitor.Width;
      var isFullHeight = location.Height == monitor.Height;

      if (!(isFullWidth || isFullHeight))
        return false;

      // Whether window is below or above the monitor's working area.
      var isHorizontalBar = (isFullWidth && location.Y + location.Height <= monitor.Y)
        || (isFullWidth && location.Y >= monitor.Y + monitor.Height);

      // Whether window is to the left or right of the monitor's working area.
      var isVerticalBar = (isFullHeight && location.X + location.Width <= monitor.X)
        || (isFullHeight && location.X >= monitor.X + monitor.Width);

      return isHorizontalBar || isVerticalBar;
    }
  }
}
