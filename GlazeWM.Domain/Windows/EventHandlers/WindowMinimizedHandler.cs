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
    private Bus _bus;
    private WindowService _windowService;
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;

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

      if (window == null)
        return;

      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);
      _bus.Invoke(new MoveContainerWithinTreeCommand(window, workspace, true));

      var minimizedWindow = new MinimizedWindow(
        window.Hwnd,
        window.OriginalWidth,
        window.OriginalHeight
      );

      _bus.Invoke(new ReplaceContainerCommand(minimizedWindow, window.Parent, window.Index));

      var focusTarget = workspace.LastFocusedDescendantExcluding(minimizedWindow) ?? workspace;

      if (focusTarget is Window)
        _bus.Invoke(new FocusWindowCommand(focusTarget as Window));

      else if (focusTarget is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((focusTarget as Workspace).Name));

      _containerService.ContainersToRedraw.Add(workspace);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
