using System;
using System.Diagnostics;
using LarsWM.Domain.Containers;
using LarsWM.Infrastructure;
using LarsWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows
{
  public class Window : Container
  {
    public Guid Id = Guid.NewGuid();
    public IntPtr Hwnd { get; }
    public bool IsHidden { get; set; } = false;

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

    public bool CanLayout => !_windowService.IsHandleCloaked(Hwnd)
        && _windowService.IsHandleManageable(Hwnd);

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
