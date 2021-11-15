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
        .FirstOrDefault(window => window.Hwnd == @event.WindowHandle);

      if (window == null)
        return;

      var tilingWindow = new TilingWindow(window.Hwnd, window.FloatingPlacement)
      {
        SizePercentage = 0
      };

      _bus.Invoke(new ReplaceContainerCommand(tilingWindow, window.Parent, window.Index));

      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);

      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      // Insert the created tiling window after the last focused descendant of the workspace.
      if (insertionTarget == null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(tilingWindow, workspace, 0, true));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            tilingWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );

      _containerService.ContainersToRedraw.Add(workspace);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
