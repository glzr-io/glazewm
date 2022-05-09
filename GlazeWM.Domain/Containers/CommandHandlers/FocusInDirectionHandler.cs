using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class FocusInDirectionHandler : ICommandHandler<FocusInDirectionCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

    public FocusInDirectionHandler(Bus bus, ContainerService containerService, MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(FocusInDirectionCommand command)
    {
      var direction = command.Direction;
      var focusedContainer = _containerService.FocusedContainer;

      if (focusedContainer is FloatingWindow)
        FocusFromFloatingWindow(focusedContainer, direction);

      else
        FocusFromTilingContainer(focusedContainer, direction);

      return CommandResponse.Ok;
    }

    private void FocusFromFloatingWindow(Container focusedContainer, Direction direction)
    {
      // Cannot focus vertically from a floating window.
      if (direction is Direction.UP or Direction.DOWN)
        return;

      var focusTarget = direction == Direction.RIGHT
        ? focusedContainer.GetNextSiblingOfType(typeof(FloatingWindow))
        : focusedContainer.GetPreviousSiblingOfType(typeof(FloatingWindow));

      // Wrap if next/previous floating window is not found.
      if (focusTarget == null)
        focusTarget = direction == Direction.RIGHT
          ? focusedContainer.SelfAndSiblingsOfType(typeof(FloatingWindow)).FirstOrDefault()
          : focusedContainer.SelfAndSiblingsOfType(typeof(FloatingWindow)).LastOrDefault();

      if (focusTarget == null || focusTarget == focusedContainer)
        return;

      _bus.Invoke(new FocusWindowCommand(focusTarget as FloatingWindow));
    }

    private void FocusFromTilingContainer(Container focusedContainer, Direction direction)
    {
      var focusTarget = GetFocusTarget(focusedContainer, direction);

      if (focusTarget is Window)
        _bus.Invoke(new FocusWindowCommand(focusTarget as Window));

      else if (focusTarget is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((focusTarget as Workspace).Name));
    }

    private Container GetFocusTarget(Container focusedContainer, Direction direction)
    {
      var focusTargetWithinWorkspace = GetFocusTargetWithinWorkspace(focusedContainer, direction);

      if (focusTargetWithinWorkspace != null)
        return focusTargetWithinWorkspace;

      // If a suitable focus target isn't found in the current workspace, attempt to find
      // a workspace in the given direction.
      return GetFocusTargetOutsideWorkspace(focusedContainer, direction);
    }

    /// <summary>
    /// Attempt to find a focus target within the focused workspace. Traverse upwards from the
    /// focused container to find an adjacent container that can be focused.
    /// </summary>
    private Container GetFocusTargetWithinWorkspace(Container focusedContainer, Direction direction)
    {
      var layoutForDirection = direction.GetCorrespondingLayout();
      var focusReference = focusedContainer;

      // Traverse upwards from the focused container. Stop searching when a workspace is
      // encountered.
      while (focusReference is not Workspace)
      {
        var parent = focusReference.Parent as SplitContainer;

        if (!focusReference.HasSiblings() || parent.Layout != layoutForDirection)
        {
          focusReference = parent;
          continue;
        }

        var focusTarget = direction is Direction.UP or Direction.LEFT
          ? focusReference.GetPreviousSiblingOfType(typeof(IResizable))
          : focusReference.GetNextSiblingOfType(typeof(IResizable));

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
    private Container GetFocusTargetOutsideWorkspace(Container focusedContainer, Direction direction)
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
