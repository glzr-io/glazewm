using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class MoveFocusedWindowHandler : ICommandHandler<MoveFocusedWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;
    private MonitorService _monitorService;

    public MoveFocusedWindowHandler(Bus bus, ContainerService containerService, WorkspaceService workspaceService, MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
      _monitorService = monitorService;
    }

    public dynamic Handle(MoveFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var direction = command.Direction;

      // The ancestor that the focused window should be moved within. This may simply be the parent of
      // the focused window, or it could be an ancestor further up the tree.
      var ancestorWithLayout = GetContainerToMoveTo(focusedWindow, direction);

      // Change the layout of the workspace to `layoutForDirection`.
      if (ancestorWithLayout == null)
      {
        var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
        workspace.Layout = GetLayoutForDirection(direction);

        // TODO: Should top-level split containers invert their layouts?
        // TODO: Should a new split container be created with the inverse layout to wrap all
        // elements other than the focused window?
        // TODO: If any top-level split containers match the new layout of the workspace,
        // then flatten them.

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

        // TODO: Not sure whether this check is needed anymore.
        if (containerToSwap != null)
        {
          _bus.Invoke(new SwapContainersCommand(focusedWindow, containerToSwap));
          _bus.Invoke(new RedrawContainersCommand());
          return CommandResponse.Ok;
        }
      }

      // Traverse up from `focusedWindow` to find container where the parent is `ancestorWithLayout`. Then,
      // depending on the direction, insert before or after that container.
      var insertionReference = focusedWindow.TraverseUpEnumeration()
        .FirstOrDefault(container => container.Parent == ancestorWithLayout);

      if (insertionReference == null)
      {
        if (direction == Direction.UP || direction == Direction.LEFT)
          _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow));
        else
          _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, 0));

        _bus.Invoke(new RedrawContainersCommand());

        // Refresh state in bar of which workspace has focus.
        _bus.RaiseEvent(new FocusChangedEvent(focusedWindow));

        return CommandResponse.Ok;
      }

      if (direction == Direction.UP || direction == Direction.LEFT)
        _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, insertionReference.Index));
      else
        _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, insertionReference.Index + 1));

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private Layout GetLayoutForDirection(Direction direction)
    {
      return (direction == Direction.LEFT || direction == Direction.RIGHT)
        ? Layout.Horizontal : Layout.Vertical;
    }

    // TODO: Consider renaming to `GetAncestorToMoveWithin`.
    private SplitContainer GetContainerToMoveTo(Window focusedWindow, Direction direction)
    {
      var layoutForDirection = GetLayoutForDirection(direction);

      var ancestorWithLayout = focusedWindow.TraverseUpEnumeration()
        .Where(ancestor =>
        {
          var isMatchingLayout = (ancestor as SplitContainer)?.Layout == layoutForDirection;

          if (!isMatchingLayout)
            return false;

          // Check whether it's possible to swap the focused window with a sibling.
          if (ancestor == focusedWindow.Parent)
          {
            var isFirstElement = focusedWindow == focusedWindow.SelfAndSiblings.First();
            var isLastElement = focusedWindow == focusedWindow.SelfAndSiblings.Last();

            if (direction == Direction.UP || direction == Direction.LEFT)
              return !isFirstElement;
            else
              return !isLastElement;
          }

          return true;
        })
        .FirstOrDefault() as SplitContainer;

      if (ancestorWithLayout != null)
        return ancestorWithLayout;

      var focusedMonitor = _monitorService.GetFocusedMonitor();
      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
      return monitorInDirection?.DisplayedWorkspace;
    }
  }
}
