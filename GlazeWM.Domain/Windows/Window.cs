using System;
using System.Diagnostics;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public class Window : Container
  {
    public Guid Id = Guid.NewGuid();
    public IntPtr Hwnd { get; }
    public bool IsHidden { get; set; } = false;

    /// <summary>
    /// Whether adjustments need to be made because of DPI (eg. when moving between monitors).
    /// </summary>
    public bool HasPendingDpiAdjustment { get; set; } = false;

    private WindowService _windowService = ServiceLocator.Provider.GetRequiredService<WindowService>();
    private ContainerService _containerService = ServiceLocator.Provider.GetRequiredService<ContainerService>();

    public Window(IntPtr hwnd)
    {
      Hwnd = hwnd;
    }

    public override int Width => _containerService.CalculateWidthOfResizableContainer(this);

    public override int Height => _containerService.CalculateHeightOfResizableContainer(this);

    public override int X => _containerService.CalculateXOfResizableContainer(this);

    public override int Y => _containerService.CalculateYOfResizableContainer(this);

    public Process Process => _windowService.GetProcessOfHandle(Hwnd);

    public string ClassName => _windowService.GetClassNameOfHandle(Hwnd);

    public WindowRect Location => _windowService.GetLocationOfHandle(Hwnd);

    public string Title => _windowService.GetTitleOfHandle(Hwnd);

    public bool IsManageable => _windowService.IsWindowManageable(this);

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
