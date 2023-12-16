using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class FocusInDirectionHandler : ICommandHandler<FocusInDirectionCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

    public FocusInDirectionHandler(
      Bus bus,
      ContainerService containerService,
      MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(FocusInDirectionCommand command)
    {
      var direction = command.Direction;
      var focusedContainer = _containerService.FocusedContainer;

      var focusedWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(focusedContainer);

      // in monocle mode, there are only two directions: next/prev
      // thus we need to override the focusInDirection command
      // and use focusInCycle instead
      if (focusedWorkspace.isMonocle)
      {
        var cycleDirection = direction;
        if (direction is Direction.Left || direction is Direction.Up)
          cycleDirection = Direction.Prev;
        if (direction is Direction.Right || direction is Direction.Down)
          cycleDirection = Direction.Next;

        _bus.Invoke(new FocusInCycleCommand(
          cycleDirection
        ));
        return CommandResponse.Ok;
      }

      var focusTarget = GetFocusTarget(focusedContainer, direction);

      if (focusTarget is null || focusTarget == focusedContainer)
        return CommandResponse.Ok;

      _bus.Invoke(new SetFocusedDescendantCommand(focusTarget));
      _containerService.HasPendingFocusSync = true;

      return CommandResponse.Ok;
    }

    private Container GetFocusTarget(Container focusedContainer, Direction direction)
    {
      if (focusedContainer is FloatingWindow)
        return GetFocusTargetFromFloating(focusedContainer, direction);

      return GetFocusTargetFromTiling(focusedContainer, direction);
    }

    private static Container GetFocusTargetFromFloating(Container focusedContainer, Direction direction)
    {
      // Cannot focus vertically from a floating window.
      if (direction is Direction.Up or Direction.Down)
        return null;

      var focusTarget = direction is Direction.Right
        ? focusedContainer.NextSiblingOfType<FloatingWindow>()
        : focusedContainer.PreviousSiblingOfType<FloatingWindow>();

      if (focusTarget is not null)
        return focusTarget;

      // Wrap if next/previous floating window is not found.
      return direction is Direction.Right
        ? focusedContainer.SelfAndSiblingsOfType<FloatingWindow>().FirstOrDefault()
        : focusedContainer.SelfAndSiblingsOfType<FloatingWindow>().LastOrDefault();
    }

    private Container GetFocusTargetFromTiling(Container focusedContainer, Direction direction)
    {
      var focusTargetWithinWorkspace = GetFocusTargetWithinWorkspace(focusedContainer, direction);

      if (focusTargetWithinWorkspace != null)
        return focusTargetWithinWorkspace;

      // If a suitable focus target isn't found in the current workspace, attempt to find
      // a workspace in the given direction.
      return GetFocusTargetOutsideWorkspace(direction);
    }

    /// <summary>
    /// Attempt to find a focus target within the focused workspace. Traverse upwards from the
    /// focused container to find an adjacent container that can be focused.
    /// </summary>
    private Container GetFocusTargetWithinWorkspace(
      Container focusedContainer,
      Direction direction)
    {
      var tilingDirection = direction.GetTilingDirection();
      var focusReference = focusedContainer;

      // Traverse upwards from the focused container. Stop searching when a workspace is
      // encountered.
      while (focusReference is not Workspace)
      {
        var parent = focusReference.Parent as SplitContainer;

        if (!focusReference.HasSiblings() || parent.TilingDirection != tilingDirection)
        {
          focusReference = parent;
          continue;
        }

        var focusTarget = direction is Direction.Up or Direction.Left
          ? focusReference.PreviousSiblingOfType<IResizable>()
          : focusReference.NextSiblingOfType<IResizable>();

        if (focusTarget == null)
        {
          focusReference = parent;
          continue;
        }

        return _containerService.GetDescendantInDirection(focusTarget, direction.Inverse());
      }

      return null;
    }

    /// <summary>
    /// Attempt to find a focus target in a different workspace than the focused workspace.
    /// </summary>
    private Container GetFocusTargetOutsideWorkspace(Direction direction)
    {
      var focusedMonitor = _monitorService.GetFocusedMonitor();

      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
      var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

      if (workspaceInDirection == null)
        return null;

      return _containerService.GetDescendantInDirection(workspaceInDirection, direction.Inverse());
    }
  }
}
