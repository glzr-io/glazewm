using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public class Window : Container
  {
    public IntPtr Hwnd { get; }

    /// <summary>
    /// The placement of the window when floating. Initialized with window's placement on launch
    /// and updated on resize/move whilst floating.
    /// </summary>
    public WindowRect FloatingPlacement { get; set; }

    /// <summary>
    /// Whether adjustments need to be made because of DPI (eg. when moving between monitors).
    /// </summary>
    public bool HasPendingDpiAdjustment { get; set; } = false;

    private WindowService _windowService = ServiceLocator.Provider.GetRequiredService<WindowService>();
    private WorkspaceService _workspaceService = ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    public Window(IntPtr hwnd, WindowRect floatingPlacement)
    {
      Hwnd = hwnd;
      FloatingPlacement = floatingPlacement;
    }

    /// <summary>
    /// Windows are hidden if their parent workspace is not displayed.
    /// </summary>
    public bool IsHidden => !_workspaceService.GetWorkspaceFromChildContainer(this).IsDisplayed;

    public string ProcessName => _windowService.GetProcessOfHandle(Hwnd).ProcessName;

    public string ClassName => _windowService.GetClassNameOfHandle(Hwnd);

    public WindowRect Location => _windowService.GetLocationOfHandle(Hwnd);

    public string Title => _windowService.GetTitleOfHandle(Hwnd);

    public bool IsManageable => _windowService.IsHandleManageable(Hwnd);

    public WS WindowStyles => _windowService.GetWindowStyles(Hwnd);

    public WS_EX WindowStylesEx => _windowService.GetWindowStylesEx(Hwnd);

    public bool HasWindowStyle(WS style)
    {
      return _windowService.HandleHasWindowStyle(Hwnd, style);
    }

    public bool HasWindowExStyle(WS_EX style)
    {
      return _windowService.HandleHasWindowExStyle(Hwnd, style);
    }
  }
}
