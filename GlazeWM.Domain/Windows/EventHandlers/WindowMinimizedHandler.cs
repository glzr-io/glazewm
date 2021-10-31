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

    public WindowMinimizedHandler(Bus bus, WindowService windowService, ContainerService containerService, WorkspaceService workspaceService)
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

      var minimizedWindow = new MinimizedWindow(
        window.Hwnd,
        window.OriginalWidth,
        window.OriginalHeight
      );

      // Keep reference to the window's ancestor workspace and focus order index prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);

      _bus.Invoke(new ReplaceContainerCommand(minimizedWindow, window.Parent, window.Index));
      _bus.Invoke(new MoveContainerWithinTreeCommand(minimizedWindow, workspace));

      var containerToFocus = workspace.LastFocusedDescendant ?? workspace;

      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));

      else if (containerToFocus is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((containerToFocus as Workspace).Name));

      _containerService.ContainersToRedraw.Add(workspace);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
