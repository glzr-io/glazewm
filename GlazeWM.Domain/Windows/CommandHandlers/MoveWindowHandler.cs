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
using GlazeWM.Infrastructure.Utils;
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
      var windowToMove = command.WindowToMove;
      var direction = command.Direction;

      if (windowToMove is FloatingWindow)
      {
        MoveFloatingWindow(windowToMove as FloatingWindow, direction);
        return CommandResponse.Ok;
      }

      if (windowToMove is TilingWindow)
      {
        MoveTilingWindow(windowToMove as TilingWindow, direction);
        return CommandResponse.Ok;
      }

      return CommandResponse.Fail;
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

        return;
      }

      // Move the window into the sibling split container.
      var targetDescendant = _containerService.GetDescendantInDirection(
        siblingInDirection,
        direction.Inverse()
      );
      var targetParent = targetDescendant.Parent as SplitContainer;

      var shouldInsertAfter =
        targetParent.TilingDirection != direction.GetTilingDirection() ||
        direction == Direction.Up ||
        direction == Direction.Left;
      var insertionIndex = shouldInsertAfter ? targetDescendant.Index + 1 : targetDescendant.Index;

      _bus.Invoke(
        new MoveContainerWithinTreeCommand(windowToMove, targetParent, insertionIndex, true)
      );
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

      // Refresh state in bar of which workspace has focus.
      _bus.Emit(new FocusChangedEvent(windowToMove));
    }

    private void ChangeWorkspaceTilingDirection(Window windowToMove, Direction direction)
    {
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToMove);

      _bus.Invoke(new ChangeTilingDirectionCommand(workspace, direction.GetTilingDirection()));

      // TODO: Should probably descend into sibling if possible.
      if (HasSiblingInDirection(windowToMove, direction))
        SwapSiblingContainers(windowToMove, direction);
    }

    private void InsertIntoAncestor(
      TilingWindow windowToMove,
      Direction direction,
      Container ancestorWithTilingDirection)
    {
      // Traverse up from `windowToMove` to find container where the parent is
      // `ancestorWithTilingDirection`. Then, depending on the direction, insert before
      // or after that container.
      var insertionReference = windowToMove.Ancestors
        .FirstOrDefault(container => container.Parent == ancestorWithTilingDirection);

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

        var shouldInsertAfter =
          targetParent.TilingDirection != direction.GetTilingDirection() ||
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

        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            windowToMove,
            ancestorWithTilingDirection,
            insertionIndex,
            true
          )
        );
      }
    }

    private void MoveTilingWindow(TilingWindow windowToMove, Direction direction)
    {
      var parentHasTilingDirection =
        (windowToMove.Parent as SplitContainer).TilingDirection == direction.GetTilingDirection();

      if (parentHasTilingDirection && HasSiblingInDirection(windowToMove, direction))
      {
        SwapSiblingContainers(windowToMove, direction);
        return;
      }

      // Attempt to the move window to workspace in given direction.
      if (parentHasTilingDirection && windowToMove.Parent is Workspace)
      {
        MoveToWorkspaceInDirection(windowToMove, direction);
        return;
      }

      // The window cannot be moved within the parent container, so traverse upwards to find a
      // suitable ancestor to move to.
      var ancestorWithTilingDirection = windowToMove.Parent.Ancestors.FirstOrDefault(
        (container) =>
          (container as SplitContainer)?.TilingDirection == direction.GetTilingDirection()
      ) as SplitContainer;

      // Change the tiling direction of the workspace to tiling direction for direction.
      if (ancestorWithTilingDirection == null)
      {
        ChangeWorkspaceTilingDirection(windowToMove, direction);
        return;
      }

      InsertIntoAncestor(windowToMove, direction, ancestorWithTilingDirection);
    }

    private void MoveFloatingWindow(Window windowToMove, Direction direction)
    {
      var valueFromConfig = _userConfigService.GeneralConfig.FloatingWindowMoveAmount;

      var amount = UnitsHelper.TrimUnits(valueFromConfig);
      var units = UnitsHelper.GetUnits(valueFromConfig);
      var currentMonitor = MonitorService.GetMonitorFromChildContainer(windowToMove);

      amount = units switch
      {
        "%" => amount * currentMonitor.Width / 100,
        "ppt" => amount * currentMonitor.Width / 100,
        "px" => amount,
        // in case user only provided a number in the config;
        // TODO: somehow validate floating_window_move_amount in config on startup
        _ => amount
        // _ => throw new ArgumentException(null, nameof(amount)),
      };

      var x = windowToMove.FloatingPlacement.X;
      var y = windowToMove.FloatingPlacement.Y;

      _ = direction switch
      {
        Direction.Left => x -= amount,
        Direction.Right => x += amount,
        Direction.Up => y -= amount,
        Direction.Down => y += amount,
        _ => throw new ArgumentException(null, nameof(direction))
      };

      // Make sure grabbable space on top is always visible
      var monitorAbove = _monitorService.GetMonitorInDirection(Direction.Up, currentMonitor);
      if (y < currentMonitor.Y && monitorAbove == null)
      {
        y = currentMonitor.Y;
      }

      var newPlacement = Rect.FromXYCoordinates(x, y, windowToMove.FloatingPlacement.Width, windowToMove.FloatingPlacement.Height);
      var center = newPlacement.GetCenterPoint();

      // If new placement wants to cross monitors
      // && direction == ... is for edge case when user places window center outside a monitor with a mouse
      if ((center.X >= currentMonitor.Width + currentMonitor.X && direction == Direction.Right) ||
      (center.X < currentMonitor.X && direction == Direction.Left) ||
      (center.Y < currentMonitor.Y && direction == Direction.Up) ||
      (center.Y >= currentMonitor.Height + currentMonitor.Y && direction == Direction.Down))
      {
        var monitorInDirection = _monitorService.GetMonitorInDirection(direction, currentMonitor);
        var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

        if (workspaceInDirection == null)
        {
          return;
        }

        // Change the window's parent workspace.
        _bus.Invoke(new MoveContainerWithinTreeCommand(windowToMove, workspaceInDirection, false));
        _bus.Emit(new FocusChangedEvent(windowToMove));

        // Redrawing twice to fix weird WindowsOS dpi behaviour
        windowToMove.HasPendingDpiAdjustment = true;
      }

      windowToMove.FloatingPlacement = newPlacement;

      _containerService.ContainersToRedraw.Add(windowToMove);
    }
  }
}
