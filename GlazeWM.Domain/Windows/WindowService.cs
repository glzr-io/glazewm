using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces;
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
    public List<IntPtr> AppBarHandles { get; set; } = new();

    /// <summary>
    /// Window handles to ignored windows (ie. windows where 'ignore' command has been invoked).
    /// </summary>
    public List<IntPtr> IgnoredHandles { get; set; } = new();

    /// <summary>
    /// Handle to the desktop window.
    /// </summary>
    public IntPtr DesktopWindowHandle { get; } = GetDesktopWindowHandle();

    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

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

    public Window GetWindowByHandle(IntPtr handle)
    {
      return GetWindows().FirstOrDefault(window => window.Handle == handle);
    }

    /// <summary>
    /// Get the id of the process that created the window.
    /// </summary>
    public static Process GetProcessOfHandle(IntPtr handle)
    {
      _ = GetWindowThreadProcessId(handle, out var processId);
      return Process.GetProcessById((int)processId);
    }

    /// <summary>
    /// Get the class name of the specified window.
    /// </summary>
    public static string GetClassNameOfHandle(IntPtr handle)
    {
      // Class name is limited to 256 characters, so it's fine to use a fixed size buffer.
      var buffer = new StringBuilder(256);
      _ = GetClassName(handle, buffer, buffer.Capacity);
      return buffer.ToString();
    }

    /// <summary>
    /// Get dimensions of the bounding rectangle of the specified window.
    /// </summary>
    public static Rect GetLocationOfHandle(IntPtr handle)
    {
      var rect = new Rect();
      GetWindowRect(handle, ref rect);
      return rect;
    }

    /// <summary>
    /// Get info about the placement of the specified window.
    /// </summary>
    public static WindowPlacement GetPlacementOfHandle(IntPtr handle)
    {
      var windowPlacement = new WindowPlacement();
      GetWindowPlacement(handle, ref windowPlacement);
      return windowPlacement;
    }

    /// <summary>
    /// Get title bar text of the specified window.
    /// </summary>
    public static string GetTitleOfHandle(IntPtr handle)
    {
      var titleLength = GetWindowTextLength(handle);

      if (titleLength == 0)
        return string.Empty;

      var buffer = new StringBuilder(titleLength + 1);
      _ = GetWindowText(handle, buffer, buffer.Capacity);
      return buffer.ToString();
    }

    public static List<IntPtr> GetAllWindowHandles()
    {
      var windowHandles = new List<IntPtr>();

      EnumWindows((IntPtr handle, int _) =>
      {
        windowHandles.Add(handle);
        return true;
      }, IntPtr.Zero);

      return windowHandles;
    }

    private static IntPtr GetDesktopWindowHandle()
    {
      return GetAllWindowHandles().Find(handle =>
      {
        var className = GetClassNameOfHandle(handle);
        var process = GetProcessOfHandle(handle);
        return className == "Progman" && process.ProcessName == "explorer";
      });
    }

    public static WindowStylesEx GetWindowStylesEx(IntPtr handle)
    {
      return unchecked((WindowStylesEx)GetWindowLongPtr(handle, GWLEXSTYLE).ToInt64());
    }

    public static WindowStyles GetWindowStyles(IntPtr handle)
    {
      return unchecked((WindowStyles)GetWindowLongPtr(handle, GWLSTYLE).ToInt64());
    }

    public static bool HandleHasWindowStyle(IntPtr handle, WindowStyles style)
    {
      return (GetWindowStyles(handle) & style) != 0;
    }

    public static bool HandleHasWindowExStyle(IntPtr handle, WindowStylesEx style)
    {
      return (GetWindowStylesEx(handle) & style) != 0;
    }

    /// <summary>
    /// Whether the given handle is cloaked. For some UWP apps, `WS_VISIBLE` will be true even if
    /// the window isn't actually visible. The `DWMWA_CLOAKED` attribute is used to check whether
    /// these apps are visible.
    /// </summary>
    public static bool IsHandleCloaked(IntPtr handle)
    {
      _ = DwmGetWindowAttribute(
        handle,
        DwmWindowAttribute.Cloaked,
        out var isCloaked,
        Marshal.SizeOf(typeof(bool))
      );

      return isCloaked;
    }

    /// <summary>
    /// Whether the given handle is actually visible.
    /// </summary>
    public static bool IsHandleVisible(IntPtr handle)
    {
      return IsWindowVisible(handle) && !IsHandleCloaked(handle);
    }

    public static bool IsHandleManageable(IntPtr handle)
    {
      if (GetProcessOfHandle(handle) is not null)
      {
        var processName = GetProcessOfHandle(handle)?.ProcessName;
        var title = GetTitleOfHandle(handle);

        // TODO: Temporary fix for managing Flow Launcher until a force manage command is added.
        if (processName == "Flow.Launcher" && title == "Flow.Launcher")
          return true;
      }

      // Ignore windows that are hidden.
      if (!IsHandleVisible(handle))
        return false;

      // Ensure window is top-level (ie. not a child window). Ignore windows that cannot be focused
      // or if they're unavailable in task switcher (alt+tab menu).
      var isApplicationWindow = !HandleHasWindowStyle(handle, WindowStyles.Child)
        && !HandleHasWindowExStyle(handle, WindowStylesEx.NoActivate | WindowStylesEx.ToolWindow);

      if (!isApplicationWindow)
        return false;

      /// Some applications spawn top-level windows for menus that should be ignored. This includes
      /// the autocomplete popup in Notepad++ and title bar menu in Keepass. Although not
      /// foolproof, these can typically be identified by having an owner window and no title bar.
      var isMenuWindow = GetWindow(handle, GW.Owner) != IntPtr.Zero
        && !HandleHasWindowStyle(handle, WindowStyles.Capion);

      return !isMenuWindow;
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

    public static WindowType GetWindowType(Window window)
    {
      return window switch
      {
        TilingWindow => WindowType.Tiling,
        FloatingWindow => WindowType.Floating,
        MaximizedWindow => WindowType.Maximized,
        FullscreenWindow => WindowType.Fullscreen,
        _ => throw new ArgumentException(null, nameof(window)),
      };
    }

    /// <summary>
    /// Get container to focus after the given window is unmanaged, minimized, or moved to another
    /// workspace.
    /// </summary>
    public static Container GetFocusTargetAfterRemoval(Window removedWindow)
    {
      var parentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(removedWindow);

      // Get descendant focus order excluding the removed container.
      var descendantFocusOrder = parentWorkspace.DescendantFocusOrder.Where(
        descendant => descendant != removedWindow
      );

      // Get focus target that matches the removed window type (only for tiling/floating windows).
      var focusTargetOfType = descendantFocusOrder.FirstOrDefault(
        (descendant) =>
          (removedWindow is FloatingWindow && descendant is FloatingWindow) ||
          (removedWindow is TilingWindow && descendant is TilingWindow)
      );

      if (focusTargetOfType is not null)
        return focusTargetOfType;

      var nonMinimizedFocusTarget = descendantFocusOrder.FirstOrDefault(
        (descendant) => descendant is not MinimizedWindow
      );

      return nonMinimizedFocusTarget ??
        descendantFocusOrder.FirstOrDefault() ??
        parentWorkspace;
    }
  }
}
