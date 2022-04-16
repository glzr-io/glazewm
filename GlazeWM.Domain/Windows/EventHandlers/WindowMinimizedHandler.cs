using System;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowMinimizedHandler : IEventHandler<WindowMinimizedEvent>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

    public WindowMinimizedHandler(
      Bus bus,
      WindowService windowService,
      ContainerService containerService,
      WorkspaceService workspaceService
    )
    {
      _bus = bus;
      _windowService = windowService;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public void Handle(WindowMinimizedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null || window is MinimizedWindow)
        return;

      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);

      // Move tiling windows to be direct children of workspace (in case they aren't already).
      if (window is TilingWindow)
        Bus.Invoke(new MoveContainerWithinTreeCommand(window, workspace, true));

      var previousState = window switch
      {
        TilingWindow _ => WindowType.TILING,
        FloatingWindow _ => WindowType.FLOATING,
        MaximizedWindow _ => WindowType.MAXIMIZED,
        FullscreenWindow _ => WindowType.FULLSCREEN,
        _ => throw new ArgumentException(),
      };

      var minimizedWindow = new MinimizedWindow(
        window.Hwnd,
        window.FloatingPlacement,
        window.BorderDelta,
        previousState
      );

      Bus.Invoke(new ReplaceContainerCommand(minimizedWindow, window.Parent, window.Index));

      var focusTarget = workspace.LastFocusedDescendantExcluding(minimizedWindow) ?? workspace;

      if (focusTarget is Window)
        Bus.Invoke(new FocusWindowCommand(focusTarget as Window));

      else if (focusTarget is Workspace)
        Bus.Invoke(new FocusWorkspaceCommand((focusTarget as Workspace).Name));

      _containerService.ContainersToRedraw.Add(workspace);
      Bus.Invoke(new RedrawContainersCommand());
    }
  }
}
