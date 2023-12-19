using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class FocusInCycleHandler : ICommandHandler<FocusInCycleCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

    public FocusInCycleHandler(
      Bus bus,
      ContainerService containerService,
      MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(FocusInCycleCommand command)
    {
      var direction = command.Direction;
      var focusedContainer = _containerService.FocusedContainer;

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
      var focusTarget = direction is Direction.Next
        ? focusedContainer.NextSiblingOfType<FloatingWindow>()
        : focusedContainer.PreviousSiblingOfType<FloatingWindow>();

      return focusTarget ?? (direction is Direction.Next
        ? focusedContainer.SelfAndSiblingsOfType<FloatingWindow>().FirstOrDefault()
        : focusedContainer.SelfAndSiblingsOfType<FloatingWindow>().LastOrDefault());
    }
    private Container GetFocusTargetFromTiling(Container focusedContainer, Direction direction)
    {
      return GetFocusTargetWithinWorkspace(focusedContainer, direction)
        ?? GetFocusTargetOutsideWorkspace(direction);
    }

    private Container GetFocusTargetWithinWorkspace(
      Container focusedContainer,
      Direction direction)
    {
      var focusReference = focusedContainer;

      while (focusReference is not Workspace)
      {
        var parent = focusReference.Parent as SplitContainer;

        if (!focusReference.HasSiblings())
        {
          focusReference = parent;
          continue;
        }

        var focusTarget = direction is Direction.Prev
          ? focusReference.PreviousSiblingOfType<IResizable>()
          : focusReference.NextSiblingOfType<IResizable>();

        if (focusTarget == null)
        {
          focusReference = parent;
          continue;
        }

        if (focusTarget is SplitContainer)
          return _containerService.GetDescendantInCycle(focusTarget, direction);

        return focusTarget;
      }
      return _containerService.GetDescendantInCycle(focusReference, direction);
    }
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
