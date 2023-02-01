using System;
using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class MoveWindowHandler : ICommandHandler<MoveWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;
    private readonly UserConfigService _userConfigService;

    public MoveWindowHandler(
      Bus bus,
      ContainerService containerService,
      MonitorService monitorService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(MoveWindowCommand command)
    {
      var wwindowToMove = command.WindowToMove;
      var direction = command.Direction;

      if (wwindowToMove is FloatingWindow)
      {
        MoveFloatingWindow(wwindowToMove, direction);
        return CommandResponse.Ok;
      }
      //move everything below to MoveTilingWindow()
      //do MoveTilingWindow(windowToMove as TilingWindow, direction)???

      var windowToMove = command.WindowToMove as TilingWindow;

      // Ignore cases where window is not tiling.
      if (windowToMove is null)
        return CommandResponse.Ok;

      var layoutForDirection = direction.GetCorrespondingLayout();
      var parentMatchesLayout =
        (windowToMove.Parent as SplitContainer).Layout == direction.GetCorrespondingLayout();

      if (parentMatchesLayout && HasSiblingInDirection(windowToMove, direction))
      {
        SwapSiblingContainers(windowToMove, direction);
        return CommandResponse.Ok;
      }

      // Attempt to the move window to workspace in given direction.
      if (parentMatchesLayout && windowToMove.Parent is Workspace)
      {
        MoveToWorkspaceInDirection(windowToMove, direction);
        return CommandResponse.Ok;
      }

      // The window cannot be moved within the parent container, so traverse upwards to find a
      // suitable ancestor to move to.
      var ancestorWithLayout = windowToMove.Parent.Ancestors.FirstOrDefault(
        container => (container as SplitContainer)?.Layout == layoutForDirection
      ) as SplitContainer;

      // Change the layout of the workspace to layout for direction.
      if (ancestorWithLayout == null)
      {
        ChangeWorkspaceLayout(windowToMove, direction);
        return CommandResponse.Ok;
      }

      InsertIntoAncestor(windowToMove, direction, ancestorWithLayout);

      return CommandResponse.Ok;
    }

    /// <summary>
    /// Whether the window has a tiling sibling in the given direction.
    /// </summary>
    private static bool HasSiblingInDirection(Window windowToMove, Direction direction)
    {
      if (direction is Direction.Up or Direction.Left)
        return windowToMove != windowToMove.SelfAndSiblingsOfType<IResizable>().First();

      return windowToMove != windowToMove.SelfAndSiblingsOfType<IResizable>().Last();
    }

    private void SwapSiblingContainers(Window windowToMove, Direction direction)
    {
      var siblingInDirection = direction is Direction.Up or Direction.Left
        ? windowToMove.PreviousSiblingOfType<IResizable>()
        : windowToMove.NextSiblingOfType<IResizable>();

      // Swap the window with sibling in given direction.
      if (siblingInDirection is Window)
      {
        var targetIndex = direction is Direction.Up or Direction.Left ?
          siblingInDirection.Index : siblingInDirection.Index + 1;

        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            windowToMove,
            windowToMove.Parent,
            targetIndex,
            false
          )
        );

        _bus.Invoke(new RedrawContainersCommand());
        return;
      }

      // Move the window into the sibling split container.
      var targetDescendant = _containerService.GetDescendantInDirection(
        siblingInDirection,
        direction.Inverse()
      );
      var targetParent = targetDescendant.Parent as SplitContainer;

      var layoutForDirection = direction.GetCorrespondingLayout();
      var shouldInsertAfter =
        targetParent.Layout != layoutForDirection ||
        direction == Direction.Up ||
        direction == Direction.Left;
      var insertionIndex = shouldInsertAfter ? targetDescendant.Index + 1 : targetDescendant.Index;

      _bus.Invoke(
        new MoveContainerWithinTreeCommand(windowToMove, targetParent, insertionIndex, true)
      );

      _bus.Invoke(new RedrawContainersCommand());
    }

    private void MoveToWorkspaceInDirection(Window windowToMove, Direction direction)
    {
      var monitor = MonitorService.GetMonitorFromChildContainer(windowToMove);
      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, monitor);
      var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

      if (workspaceInDirection == null)
        return;

      // Since window is crossing monitors, adjustments might need to be made because of DPI.
      if (MonitorService.HasDpiDifference(windowToMove, workspaceInDirection))
        windowToMove.HasPendingDpiAdjustment = true;

      // Update floating placement since the window has to cross monitors.
      windowToMove.FloatingPlacement =
        windowToMove.FloatingPlacement.TranslateToCenter(workspaceInDirection.ToRect());

      // TODO: Descend into container if possible.
      if (direction is Direction.Up or Direction.Left)
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, workspaceInDirection, true));
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, workspaceInDirection, 0, true));

      _bus.Invoke(new RedrawContainersCommand());

      // Refresh state in bar of which workspace has focus.
      _bus.Emit(new FocusChangedEvent(windowToMove));
    }

    private void ChangeWorkspaceLayout(Window windowToMove, Direction direction)
    {
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToMove);

      var layoutForDirection = direction.GetCorrespondingLayout();
      _bus.Invoke(new ChangeContainerLayoutCommand(workspace, layoutForDirection));

      // TODO: Should probably descend into sibling if possible.
      if (HasSiblingInDirection(windowToMove, direction))
        SwapSiblingContainers(windowToMove, direction);

      _bus.Invoke(new RedrawContainersCommand());
    }

    private void InsertIntoAncestor(
      TilingWindow windowToMove,
      Direction direction,
      Container ancestorWithLayout)
    {
      // Traverse up from `windowToMove` to find container where the parent is `ancestorWithLayout`.
      // Then, depending on the direction, insert before or after that container.
      var insertionReference = windowToMove.Ancestors
        .FirstOrDefault(container => container.Parent == ancestorWithLayout);

      var insertionReferenceSibling = direction is Direction.Up or Direction.Left
        ? insertionReference.PreviousSiblingOfType<IResizable>()
        : insertionReference.NextSiblingOfType<IResizable>();

      if (insertionReferenceSibling is SplitContainer)
      {
        // Move the window into the adjacent split container.
        var targetDescendant = _containerService.GetDescendantInDirection(
          insertionReferenceSibling,
          direction.Inverse()
        );
        var targetParent = targetDescendant.Parent as SplitContainer;

        var layoutForDirection = direction.GetCorrespondingLayout();
        var shouldInsertAfter =
          targetParent.Layout != layoutForDirection ||
          direction == Direction.Up ||
          direction == Direction.Left;

        var insertionIndex = shouldInsertAfter
          ? targetDescendant.Index + 1
          : targetDescendant.Index;

        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, targetParent, insertionIndex, true));
      }
      else
      {
        // Move the window into the container above.
        var insertionIndex = (direction is Direction.Up or Direction.Left) ?
          insertionReference.Index : insertionReference.Index + 1;

        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, ancestorWithLayout, insertionIndex, true));
      }

      _bus.Invoke(new RedrawContainersCommand());
    }

    private void MoveFloatingWindow(Window windowToMove, Direction direction)
    {
      int amount = _userConfigService.GeneralConfig.FloatingWindowMoveAmount;

      var x = windowToMove.FloatingPlacement.X;
      var y = windowToMove.FloatingPlacement.Y;

      switch (direction)
      {
        case Direction.Left:
          x -= amount;
          break;

        case Direction.Right:
          x += amount;
          break;

        case Direction.Up:
          y -= amount;
          break;

        case Direction.Down:
          y += amount;
          break;
      }

      windowToMove.FloatingPlacement = Rect.FromXYCoordinates(x, y, windowToMove.FloatingPlacement.Width, windowToMove.FloatingPlacement.Height);

      _containerService.ContainersToRedraw.Add(windowToMove);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
