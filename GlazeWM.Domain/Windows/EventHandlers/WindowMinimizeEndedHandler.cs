using System;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  class WindowMinimizeEndedHandler : IEventHandler<WindowMinimizeEndedEvent>
  {
    private Bus _bus;
    private WindowService _windowService;
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;

    public WindowMinimizeEndedHandler(
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

    public void Handle(WindowMinimizeEndedEvent @event)
    {
      var window = _windowService.GetWindows()
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle) as MinimizedWindow;

      if (window == null)
        return;

      var restoredWindow = CreateWindowFromPreviousState(window);

      _bus.Invoke(new ReplaceContainerCommand(restoredWindow, window.Parent, window.Index));

      if (!(restoredWindow is TilingWindow))
        return;

      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);
      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      // Insert the created tiling window after the last focused descendant of the workspace.
      if (insertionTarget == null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(restoredWindow, workspace, 0, true));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            restoredWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );

      _containerService.ContainersToRedraw.Add(workspace);
      _bus.Invoke(new RedrawContainersCommand());
    }

    private Window CreateWindowFromPreviousState(MinimizedWindow window)
    {
      return window.PreviousState switch
      {
        WindowType.FLOATING => new FloatingWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta
        ),
        WindowType.TILING => new TilingWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta
        )
        { SizePercentage = 0 },
        WindowType.MAXIMIZED => new MaximizedWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta
        ),
        WindowType.FULLSCREEN => new FullscreenWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta
        ),
        _ => throw new ArgumentException(),
      };
    }
  }
}
