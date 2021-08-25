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
      var layoutForDirection = GetLayoutForDirection(direction);
      var parentMatchesLayout = (focusedWindow.Parent as SplitContainer).Layout == layoutForDirection;

      // Swap the focused window with sibling in given direction.
      if (parentMatchesLayout && HasSiblingInDirection(focusedWindow, direction))
      {
        var containerToSwap = (direction == Direction.UP || direction == Direction.LEFT) ?
          focusedWindow.SelfAndSiblings[focusedWindow.Index - 1]
          : focusedWindow.SelfAndSiblings[focusedWindow.Index + 1];

        // TODO: If container to swap is not a `Window`, then descend into sibling
        // container and insert at end of focus stack.

        _bus.Invoke(new SwapContainersCommand(focusedWindow, containerToSwap));
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      // Attempt to the move focused window to workspace in given direction.
      if (parentMatchesLayout && focusedWindow.Parent is Workspace)
      {
        var focusedMonitor = _monitorService.GetFocusedMonitor();
        var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
        var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

        if (workspaceInDirection == null)
          return CommandResponse.Ok;

        if (direction == Direction.UP || direction == Direction.LEFT)
          _bus.Invoke(new AttachContainerCommand(workspaceInDirection, focusedWindow));
        else
          _bus.Invoke(new AttachContainerCommand(workspaceInDirection, focusedWindow, 0));

        _bus.Invoke(new RedrawContainersCommand());

        // Refresh state in bar of which workspace has focus.
        _bus.RaiseEvent(new FocusChangedEvent(focusedWindow));

        return CommandResponse.Ok;
      }

      // The ancestor that the focused window should be moved within. This may simply be the parent of
      // the focused window, or it could be an ancestor further up the tree.
      // Since focused window cannot be moved within the parent container, traverse upwards to find
      // a suitable ancestor to move to.
      var ancestorWithLayout = focusedWindow.Parent.TraverseUpEnumeration()
        .Where(container => (container as SplitContainer)?.Layout == layoutForDirection)
        .FirstOrDefault() as SplitContainer;

      // Change the layout of the workspace to `layoutForDirection`.
      if (ancestorWithLayout == null)
      {
        var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
        workspace.Layout = GetLayoutForDirection(direction);

        // TODO: Should top-level split containers invert their layouts?
        // TODO: Should a new split container be created with the inverse layout to wrap all
        // elements other than the focused window?
        // TODO: Flatten any top-level split containers with the changed layout of the workspace.

        _containerService.SplitContainersToRedraw.Add(workspace);
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      // Traverse up from `focusedWindow` to find container where the parent is `ancestorWithLayout`. Then,
      // depending on the direction, insert before or after that container.
      var insertionReference = focusedWindow.TraverseUpEnumeration()
        .FirstOrDefault(container => container.Parent == ancestorWithLayout);

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

    /// <summary>
    /// Whether the focused window has a sibling in the given direction.
    /// </summary>
    private bool HasSiblingInDirection(Window focusedWindow, Direction direction)
    {
      if (direction == Direction.UP || direction == Direction.LEFT)
        return focusedWindow != focusedWindow.SelfAndSiblings.First();
      else
        return focusedWindow != focusedWindow.SelfAndSiblings.Last();
    }
  }
}
