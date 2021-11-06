using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
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

    public CommandResponse Handle(MoveFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as TilingWindow;

      // Ignore cases where focused container is not a tiling window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var direction = command.Direction;
      var layoutForDirection = direction.GetCorrespondingLayout();
      var parentMatchesLayout = (focusedWindow.Parent as SplitContainer).Layout == direction.GetCorrespondingLayout();

      if (parentMatchesLayout && HasSiblingInDirection(focusedWindow, direction))
      {
        SwapSiblingContainers(focusedWindow, direction);
        return CommandResponse.Ok;
      }

      // Attempt to the move focused window to workspace in given direction.
      if (parentMatchesLayout && focusedWindow.Parent is Workspace)
      {
        MoveToWorkspaceInDirection(focusedWindow, direction);
        return CommandResponse.Ok;
      }

      // The focused window cannot be moved within the parent container, so traverse upwards to find
      // a suitable ancestor to move to.
      var ancestorWithLayout = focusedWindow.Parent.TraverseUpEnumeration()
        .Where(container => (container as SplitContainer)?.Layout == layoutForDirection)
        .FirstOrDefault() as SplitContainer;

      // Change the layout of the workspace to layout for direction.
      if (ancestorWithLayout == null)
      {
        ChangeWorkspaceLayout(focusedWindow, direction);
        return CommandResponse.Ok;
      }

      InsertIntoAncestor(focusedWindow, direction, ancestorWithLayout);

      return CommandResponse.Ok;
    }

    /// <summary>
    /// Whether the focused window has a tiling sibling in the given direction.
    /// </summary>
    private bool HasSiblingInDirection(Window focusedWindow, Direction direction)
    {
      if (direction == Direction.UP || direction == Direction.LEFT)
        return focusedWindow != focusedWindow.SelfAndSiblingsOfType(typeof(IResizable)).First();
      else
        return focusedWindow != focusedWindow.SelfAndSiblingsOfType(typeof(IResizable)).Last();
    }

    private void SwapSiblingContainers(Window focusedWindow, Direction direction)
    {
      var siblingInDirection = direction == Direction.UP || direction == Direction.LEFT
        ? focusedWindow.GetPreviousSiblingOfType(typeof(IResizable))
        : focusedWindow.GetNextSiblingOfType(typeof(IResizable));

      // Swap the focused window with sibling in given direction.
      if (siblingInDirection is Window)
      {
        var targetIndex = direction == Direction.UP || direction == Direction.LEFT ?
          siblingInDirection.Index : siblingInDirection.Index + 1;

        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            focusedWindow,
            focusedWindow.Parent,
            targetIndex,
            false
          )
        );

        _bus.Invoke(new RedrawContainersCommand());
        return;
      }

      // Move the focused window into the sibling split container.
      var targetDescendant = _containerService.GetDescendantInDirection(siblingInDirection, direction.Inverse());
      var targetParent = targetDescendant.Parent as SplitContainer;

      var layoutForDirection = direction.GetCorrespondingLayout();
      var insertionIndex = targetParent.Layout != layoutForDirection || direction == Direction.UP ||
        direction == Direction.LEFT ? targetDescendant.Index + 1 : targetDescendant.Index;

      _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, targetParent, insertionIndex, true));
      _bus.Invoke(new RedrawContainersCommand());
    }

    private void MoveToWorkspaceInDirection(Window focusedWindow, Direction direction)
    {
      var focusedMonitor = _monitorService.GetFocusedMonitor();
      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
      var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

      if (workspaceInDirection == null)
        return;

      // Since window is crossing monitors, adjustments might need to be made because of DPI.
      if (_monitorService.HasDpiDifference(focusedWindow, workspaceInDirection))
        focusedWindow.HasPendingDpiAdjustment = true;

      // TODO: Descend into container if possible.
      if (direction == Direction.UP || direction == Direction.LEFT)
        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, workspaceInDirection, true));
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, workspaceInDirection, 0, true));

      _bus.Invoke(new RedrawContainersCommand());

      // Refresh state in bar of which workspace has focus.
      _bus.RaiseEvent(new FocusChangedEvent(focusedWindow));
    }

    private void ChangeWorkspaceLayout(Window focusedWindow, Direction direction)
    {
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(focusedWindow);

      var layoutForDirection = direction.GetCorrespondingLayout();
      _bus.Invoke(new ChangeContainerLayoutCommand(workspace, layoutForDirection));

      // TODO: Should probably descend into sibling if possible.
      if (HasSiblingInDirection(focusedWindow, direction))
        SwapSiblingContainers(focusedWindow, direction);

      _bus.Invoke(new RedrawContainersCommand());
    }

    private void InsertIntoAncestor(TilingWindow focusedWindow, Direction direction, Container ancestorWithLayout)
    {
      // Traverse up from `focusedWindow` to find container where the parent is `ancestorWithLayout`. Then,
      // depending on the direction, insert before or after that container.
      var insertionReference = focusedWindow.TraverseUpEnumeration()
        .FirstOrDefault(container => container.Parent == ancestorWithLayout);

      var insertionReferenceSibling = direction == Direction.UP || direction == Direction.LEFT
        ? insertionReference.GetPreviousSiblingOfType(typeof(IResizable))
        : insertionReference.GetNextSiblingOfType(typeof(IResizable));

      if (insertionReferenceSibling is SplitContainer)
      {
        // Move the focused window into the adjacent split container.
        var targetDescendant = _containerService.GetDescendantInDirection(insertionReferenceSibling, direction.Inverse());
        var targetParent = targetDescendant.Parent as SplitContainer;

        var layoutForDirection = direction.GetCorrespondingLayout();
        var insertionIndex = targetParent.Layout != layoutForDirection || direction == Direction.UP ||
          direction == Direction.LEFT ? targetDescendant.Index + 1 : targetDescendant.Index;

        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, targetParent, insertionIndex, true));
      }
      else
      {
        // Move the focused window into the container above.
        var insertionIndex = (direction == Direction.UP || direction == Direction.LEFT) ?
          insertionReference.Index : insertionReference.Index + 1;

        _bus.Invoke(new MoveContainerWithinTreeCommand(focusedWindow, ancestorWithLayout, insertionIndex, true));
      }

      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
