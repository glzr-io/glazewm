using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
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
      var focusedWindow = _containerService.FocusedContainer as Window;
      var foregroundWindow = GetForegroundWindow();

      // TODO: Allow command to be called from a focused workspace with no children.

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow == null || foregroundWindow != focusedWindow.Hwnd)
        return CommandResponse.Ok;

      var focusTarget = GetFocusTarget(focusedWindow, direction);

      if (focusTarget is Window)
        _bus.Invoke(new FocusWindowCommand(focusTarget as Window));

      else if (focusTarget is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((focusTarget as Workspace).Name));

      return CommandResponse.Ok;
    }

    private Container GetFocusTarget(Window focusedWindow, Direction direction)
    {
      var layoutForDirection = direction.GetCorrespondingLayout();

      Container focusTargetRef = focusedWindow;

      // Attempt to find a focus target within the current workspace by traversing upwards from
      // the focused container.
      while (!(focusTargetRef is Workspace))
      {
        var parent = focusTargetRef.Parent as SplitContainer;

        if (!focusTargetRef.HasSiblings() || parent.Layout != layoutForDirection)
        {
          focusTargetRef = focusTargetRef.Parent;
          continue;
        }

        var focusTarget = direction == Direction.UP || direction == Direction.LEFT ?
          focusTargetRef.PreviousSibling : focusTargetRef.NextSibling;

        if (focusTarget == null)
        {
          focusTargetRef = focusTargetRef.Parent;
          continue;
        }

        if (focusTarget is SplitContainer)
          return _containerService.GetDescendantInDirection(focusTarget, direction.Inverse());

        return focusTarget;
      }

      var focusedMonitor = _monitorService.GetFocusedMonitor();

      // If a suitable focus target isn't found in the current workspace, attempt to find
      // a workspace in the given direction.
      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
      var workspaceInDirection = monitorInDirection?.DisplayedWorkspace;

      if (workspaceInDirection == null)
        return null;

      return _containerService.GetDescendantInDirection(workspaceInDirection, direction.Inverse());
    }
  }
}
