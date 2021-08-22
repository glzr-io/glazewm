using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class MoveFocusedWindowHandler : ICommandHandler<MoveFocusedWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;

    public MoveFocusedWindowHandler(Bus bus, ContainerService containerService, WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public dynamic Handle(MoveFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var direction = command.Direction;
      var layoutForDirection = (direction == Direction.LEFT || direction == Direction.RIGHT)
        ? Layout.Horizontal : Layout.Vertical;

      var ancestorWithLayout = focusedWindow.TraverseUpEnumeration().Where(con => (con as SplitContainer)?.Layout == layoutForDirection).FirstOrDefault();

      // Change the layout of the workspace to `layoutForDirection`.
      if (ancestorWithLayout == null)
      {
        var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
        workspace.Layout = layoutForDirection;

        _containerService.SplitContainersToRedraw.Add(workspace);
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      return CommandResponse.Ok;
    }
  }
}
