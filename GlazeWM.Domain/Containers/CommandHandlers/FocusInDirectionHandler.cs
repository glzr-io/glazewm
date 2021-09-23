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
  class FocusInDirectionHandler : ICommandHandler<FocusInDirectionCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private MonitorService _monitorService;

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

      var focusTarget = GetFocusTarget(focusedContainer, direction);

      if (focusTarget is Window)
        _bus.Invoke(new FocusWindowCommand(focusTarget as Window));

      else if (focusTarget is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((focusTarget as Workspace).Name));

      return CommandResponse.Ok;
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

      while (!(focusReference is Workspace))
      {
        var parent = focusReference.Parent as SplitContainer;

        if (!focusReference.HasSiblings() || parent.Layout != layoutForDirection)
        {
          focusReference = parent;
          continue;
        }

        var focusTarget = direction == Direction.UP || direction == Direction.LEFT ?
          focusReference.PreviousSibling : focusReference.NextSibling;

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
