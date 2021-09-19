using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using LarsWM.Domain.Containers;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.WindowsApi;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows
{
  public class WindowService
  {
    private ContainerService _containerService;
    private UserConfigService _userConfigService;

    public WindowService(ContainerService containerService, UserConfigService userConfigService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    /// <summary>
    /// Get windows by searching entire container forest for Window containers.
    /// </summary>
    public IEnumerable<Window> GetWindows()
    {
      return _containerService.ContainerTree.TraverseDownEnumeration()
        .OfType<Window>();
    }

    /// <summary>
    /// Get windows within given parent container.
    /// </summary>
    public IEnumerable<Window> GetWindowsOfParentContainer(Container parent)
    {
      return parent.TraverseDownEnumeration()
        .OfType<Window>();
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
      WindowRect rect = new WindowRect();
      GetWindowRect(handle, ref rect);
      return rect;
    }

    /// <summary>
    /// Get title bar text of the specified window.
    /// </summary>
    public string GetTitleOfHandle(IntPtr handle)
    {
      int titleLength = GetWindowTextLength(handle);

      if (titleLength == 0)
        return String.Empty;

      var buffer = new StringBuilder(titleLength + 1);
      GetWindowText(handle, buffer, buffer.Capacity);
      return buffer.ToString();
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

    public bool IsHandleCloaked(IntPtr handle)
    {

      bool isCloaked;
      DwmGetWindowAttribute(handle, DwmWindowAttribute.DWMWA_CLOAKED, out isCloaked, Marshal.SizeOf(typeof(bool)));
      return isCloaked;
    }

    public bool IsWindowManageable(Window window)
    {
      var isApplicationWindow = IsWindowVisible(window.Hwnd)
          && !window.HasWindowStyle(WS.WS_CHILD) && !window.HasWindowExStyle(WS_EX.WS_EX_NOACTIVATE);

      var isCurrentProcess = window.Process.Id == Process.GetCurrentProcess().Id;

      var isExcludedClassName = _userConfigService.UserConfig.WindowClassesToIgnore.Contains(window.ClassName);
      var isExcludedProcessName = _userConfigService.UserConfig.ProcessNamesToIgnore.Contains(window.Process.ProcessName);

      var isShellWindow = window.Hwnd == GetShellWindow();

      if (isApplicationWindow && !isCurrentProcess && !isExcludedClassName && !isExcludedProcessName && !isShellWindow)
      {
        return true;
      }

      return false;
    }

    // TODO: Merge with IsWindowManageable method.
    public bool IsHandleManageable(IntPtr handle)
    {

      if (HandleHasWindowExStyle(handle, WS_EX.WS_EX_TOOLWINDOW) ||
          GetWindow(handle, GW.GW_OWNER) != IntPtr.Zero)
      {
        return false;
      }

      return true;
    }
  }
}
