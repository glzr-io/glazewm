using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ToggleFloatingHandler : ICommandHandler<ToggleFloatingCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private WindowService _windowService;
    private ContainerService _containerService;

    public ToggleFloatingHandler(Bus bus, WorkspaceService workspaceService, WindowService windowService, ContainerService containerService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _windowService = windowService;
      _containerService = containerService;
    }

    public CommandResponse Handle(ToggleFloatingCommand command)
    {
      var window = command.Window;

      if (window is FloatingWindow)
        DisableFloating(window as FloatingWindow);

      else
        EnableFloating(window);

      return CommandResponse.Ok;
    }

    private void EnableFloating(Window window)
    {
      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);

      _bus.Invoke(new MoveContainerWithinTreeCommand(window, workspace, true));

      // Create a floating window and place it in the center of the workspace.
      var floatingWindow = new FloatingWindow(
        window.Hwnd,
        window.FloatingPlacement,
        window.BorderDelta
      );

      _bus.Invoke(new ReplaceContainerCommand(floatingWindow, window.Parent, window.Index));
      _bus.Invoke(new RedrawContainersCommand());
    }

    private void DisableFloating(FloatingWindow floatingWindow)
    {
      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(floatingWindow);

      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      var tilingWindow = new TilingWindow(floatingWindow.Hwnd, floatingWindow.FloatingPlacement, floatingWindow.BorderDelta)
      {
        SizePercentage = 0
      };

      _bus.Invoke(new ReplaceContainerCommand(tilingWindow, floatingWindow.Parent, floatingWindow.Index));

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

      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
