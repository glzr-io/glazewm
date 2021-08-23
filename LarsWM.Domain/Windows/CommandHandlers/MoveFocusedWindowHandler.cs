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

      // The ancestor that the focused window should be moved within. This may simply be the parent of
      // the focused window, or it could be an ancestor further up the tree.
      var ancestorWithLayout = focusedWindow.TraverseUpEnumeration()
        .Where(container => (container as SplitContainer)?.Layout == layoutForDirection)
        .FirstOrDefault() as SplitContainer;

      // Change the layout of the workspace to `layoutForDirection`.
      if (ancestorWithLayout == null)
      {
        var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
        workspace.Layout = layoutForDirection;

        _containerService.SplitContainersToRedraw.Add(workspace);
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      // Swap the focused window with sibling in given direction.
      if (ancestorWithLayout == focusedWindow.Parent)
      {
        var index = focusedWindow.Index;

        var containerToSwap = (direction == Direction.UP || direction == Direction.LEFT) ?
          focusedWindow.SelfAndSiblings.ElementAtOrDefault(index - 1)
          : focusedWindow.SelfAndSiblings.ElementAtOrDefault(index + 1);

        if (containerToSwap != null)
        {
          _bus.Invoke(new SwapContainersCommand(focusedWindow, containerToSwap));
          _bus.Invoke(new RedrawContainersCommand());
          return CommandResponse.Ok;
        }
      }

      // Traverse up from `focusedWindow` to find container where the parent is `ancestorWithLayout`. Then,
      // depending on the direction, insert before or after that container.
      var insertionIndex = focusedWindow.TraverseUpEnumeration()
        .FirstOrDefault(container => container.Parent == ancestorWithLayout)
        .Index;

      if (direction == Direction.UP || direction == Direction.LEFT)
        _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, insertionIndex));
      else
        _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, insertionIndex + 1));

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
