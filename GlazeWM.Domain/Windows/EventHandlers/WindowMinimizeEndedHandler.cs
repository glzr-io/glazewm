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
    private readonly Bus _bus;
    private readonly WindowService _windowService;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

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

      Bus.Invoke(new ReplaceContainerCommand(restoredWindow, window.Parent, window.Index));

      if (restoredWindow is not TilingWindow)
        return;

      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);
      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      // Insert the created tiling window after the last focused descendant of the workspace.
      if (insertionTarget == null)
        Bus.Invoke(new MoveContainerWithinTreeCommand(restoredWindow, workspace, 0, true));
      else
        Bus.Invoke(
          new MoveContainerWithinTreeCommand(
            restoredWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );

      _containerService.ContainersToRedraw.Add(workspace);
      Bus.Invoke(new RedrawContainersCommand());
    }

    private static Window CreateWindowFromPreviousState(MinimizedWindow window)
    {
      return window.PreviousState switch
      {
        WindowType.FLOATING => new FloatingWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta
        ),
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
        // Set `SizePercentage` to 0 to correctly resize the container when moved within tree.
        WindowType.TILING => new TilingWindow(
          window.Hwnd,
          window.FloatingPlacement,
          window.BorderDelta,
          0
        ),
        _ => throw new ArgumentException(),
      };
    }
  }
}
