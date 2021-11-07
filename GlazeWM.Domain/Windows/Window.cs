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
    /// Whether adjustments need to be made because of DPI (eg. when moving between monitors).
    /// </summary>
    public bool HasPendingDpiAdjustment { get; set; } = false;

    /// <summary>
    /// The original width of the window. Used as the window's width when floating is toggled.
    /// </summary>
    public int OriginalWidth { get; set; }

    /// <summary>
    /// The original height of the window. Used as the window's height when floating is toggled.
    /// </summary>
    public int OriginalHeight { get; set; }

    private WindowService _windowService = ServiceLocator.Provider.GetRequiredService<WindowService>();
    private WorkspaceService _workspaceService = ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    public Window(IntPtr hwnd, int originalWidth, int originalHeight)
    {
      Hwnd = hwnd;
      OriginalWidth = originalWidth;
      OriginalHeight = originalHeight;
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
