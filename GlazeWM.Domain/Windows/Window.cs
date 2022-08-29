using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.WindowsApi;
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
    /// The difference in window dimensions to adjust for invisible borders. This is typically 7px
    /// on the left, right, and bottom edges. This needs to be adjusted for to draw a window with
    /// exact dimensions.
    /// </summary>
    public RectDelta BorderDelta { get; set; } = new RectDelta(7, 0, 7, 7);

    /// <summary>
    /// Whether adjustments need to be made because of DPI (eg. when moving between monitors).
    /// </summary>
    public bool HasPendingDpiAdjustment { get; set; }

    public Window(IntPtr hwnd, WindowRect floatingPlacement, RectDelta borderDelta)
    {
      Hwnd = hwnd;
      FloatingPlacement = floatingPlacement;
      BorderDelta = borderDelta;
    }

    /// <summary>
    /// Windows are displayed if their parent workspace is displayed.
    /// </summary>
    public bool IsDisplayed => WorkspaceService.GetWorkspaceFromChildContainer(this).IsDisplayed;

    public string ProcessName => WindowService.GetProcessOfHandle(Hwnd)?.ProcessName ?? string.Empty;

    public string ClassName => WindowService.GetClassNameOfHandle(Hwnd);

    public WindowRect Location => WindowService.GetLocationOfHandle(Hwnd);

    public string Title => WindowService.GetTitleOfHandle(Hwnd);

    public bool IsManageable => WindowService.IsHandleManageable(Hwnd);

    public WS WindowStyles => WindowService.GetWindowStyles(Hwnd);

    public WS_EX WindowStylesEx => WindowService.GetWindowStylesEx(Hwnd);

    public bool HasWindowStyle(WS style)
    {
      return WindowService.HandleHasWindowStyle(Hwnd, style);
    }

    public bool HasWindowExStyle(WS_EX style)
    {
      return WindowService.HandleHasWindowExStyle(Hwnd, style);
    }
  }
}
