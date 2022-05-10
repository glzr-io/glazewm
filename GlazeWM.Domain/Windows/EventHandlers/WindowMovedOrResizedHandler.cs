using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal class WindowMovedOrResizedHandler : IEventHandler<WindowMovedOrResizedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly MonitorService _monitorService;

    public WindowMovedOrResizedHandler(
      Bus bus,
      WindowService windowService,
      MonitorService monitorService
    )
    {
      _bus = bus;
      _windowService = windowService;
      _monitorService = monitorService;
    }

    public void Handle(WindowMovedOrResizedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window is null or not FloatingWindow)
        return;

      // Update state with new location of the floating window.
      UpdateWindowPlacement(window);

      // Change floating window's parent workspace if out of its bounds.
      UpdateParentWorkspace(window);
    }

    private static void UpdateWindowPlacement(Window window)
    {
      window.FloatingPlacement = WindowService.GetPlacementOfHandle(window.Hwnd).NormalPosition;
    }

    private void UpdateParentWorkspace(Window window)
    {
      var currentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(window);

      // Get workspace that encompasses most of the window.
      var targetMonitor = _monitorService.GetMonitorFromHandleLocation(window.Hwnd);
      var targetWorkspace = targetMonitor.DisplayedWorkspace;

      // Ignore if window is still within the bounds of its current workspace.
      if (currentWorkspace == targetWorkspace)
        return;

      // Change the window's parent workspace.
      _bus.Invoke(new MoveContainerWithinTreeCommand(window, targetWorkspace, false));
      _bus.RaiseEvent(new FocusChangedEvent(window));
    }
  }
}
