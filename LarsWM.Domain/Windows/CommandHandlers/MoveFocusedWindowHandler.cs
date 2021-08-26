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
      var layoutForDirection = direction.GetCorrespondingLayout();
      var parentMatchesLayout = (focusedWindow.Parent as SplitContainer).Layout == layoutForDirection;

      if (parentMatchesLayout && HasSiblingInDirection(focusedWindow, direction))
      {
        var siblingInDirection = (direction == Direction.UP || direction == Direction.LEFT) ?
          focusedWindow.PreviousSibling : focusedWindow.NextSibling;

        // Swap the focused window with sibling in given direction.
        if (siblingInDirection is Window)
          _bus.Invoke(new SwapContainersCommand(focusedWindow, siblingInDirection));
        else
        {
          // Move the focused window into the sibling split container.
          var targetDescendant = _containerService.GetDescendantInDirection(siblingInDirection, direction.Inverse());
          var targetParent = targetDescendant.Parent as SplitContainer;

          var insertionIndex = targetParent.Layout != layoutForDirection || direction == Direction.UP ||
            direction == Direction.LEFT ? targetDescendant.Index + 1 : targetDescendant.Index;

          _bus.Invoke(new AttachContainerCommand(targetParent, focusedWindow, insertionIndex));
        }

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

      // The focused window cannot be moved within the parent container, so traverse upwards to find
      // a suitable ancestor to move to.
      var ancestorWithLayout = focusedWindow.Parent.TraverseUpEnumeration()
        .Where(container => (container as SplitContainer)?.Layout == layoutForDirection)
        .FirstOrDefault() as SplitContainer;

      // Change the layout of the workspace to `layoutForDirection`.
      if (ancestorWithLayout == null)
      {
        var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);
        workspace.Layout = direction.GetCorrespondingLayout();

        // TODO: Flatten any top-level split containers with the changed layout of the workspace.
        // TODO: Index of focused window might have to be changed after layout is inverted.
        // TODO: Allow ChangeContainerLayoutCommand to work on workspaces and use that to change
        // the layout.

        _containerService.SplitContainersToRedraw.Add(workspace);
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      // Traverse up from `focusedWindow` to find container where the parent is `ancestorWithLayout`. Then,
      // depending on the direction, insert before or after that container.
      var insertionReference = focusedWindow.TraverseUpEnumeration()
        .FirstOrDefault(container => container.Parent == ancestorWithLayout);

      var insertionReferenceSibling = direction == Direction.UP || direction == Direction.LEFT ?
        insertionReference.PreviousSibling : insertionReference.NextSibling;

      if (insertionReferenceSibling is SplitContainer)
      {
        // Move the focused window into the adjacent split container.
        var targetDescendant = _containerService.GetDescendantInDirection(insertionReferenceSibling, direction.Inverse());
        var targetParent = targetDescendant.Parent as SplitContainer;

        var insertionIndex = targetParent.Layout != layoutForDirection || direction == Direction.UP ||
          direction == Direction.LEFT ? targetDescendant.Index + 1 : targetDescendant.Index;

        _bus.Invoke(new AttachContainerCommand(targetParent, focusedWindow, insertionIndex));
      }
      else
      {
        // Move the focused window into the container above.
        var insertionIndex = (direction == Direction.UP || direction == Direction.LEFT) ?
          insertionReference.Index : insertionReference.Index + 1;

        _bus.Invoke(new AttachContainerCommand(ancestorWithLayout, focusedWindow, insertionIndex));
      }

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
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
