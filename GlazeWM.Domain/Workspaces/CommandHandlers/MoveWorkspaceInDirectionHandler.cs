using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal sealed class MoveWorkspaceInDirectionHandler :
    ICommandHandler<MoveWorkspaceInDirectionCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;

    public MoveWorkspaceInDirectionHandler(
      Bus bus,
      MonitorService monitorService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(MoveWorkspaceInDirectionCommand command)
    {
      var direction = command.Direction;

      // Get focused workspace + monitor.
      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();
      var focusedMonitor = MonitorService.GetMonitorFromChildContainer(focusedWorkspace);

      // Get monitor in the given direction from the focused workspace.
      var targetMonitor = _monitorService.GetMonitorInDirection(
        direction,
        focusedMonitor
      );

      if (targetMonitor is null)
        return CommandResponse.Ok;

      // Move workspace to target monitor.
      _bus.Invoke(
        new MoveContainerWithinTreeCommand(focusedWorkspace, targetMonitor, false)
      );

      // Update floating placement since the windows have to cross monitors.
      foreach (var window in focusedWorkspace.Descendants.OfType<Window>())
      {
        window.FloatingPlacement = window.FloatingPlacement.TranslateToCenter(
          targetMonitor.DisplayedWorkspace.ToRect()
        );
      }

      // Prevent original monitor from having no workspaces.
      if (focusedMonitor.Children.Count == 0)
        ActivateWorkspaceOnMonitor(focusedMonitor);

      // Update workspaces displayed in bar window.
      // TODO: Consider creating separate event `WorkspaceMovedEvent`.
      _bus.Emit(new WorkspaceActivatedEvent(focusedWorkspace));

      return CommandResponse.Ok;
    }

    private void ActivateWorkspaceOnMonitor(Monitor monitor)
    {
      // Get name of first workspace that is not active for that specified monitor or any.
      var inactiveWorkspaceConfig =
        _workspaceService.GetWorkspaceConfigToActivate(monitor);

      if (inactiveWorkspaceConfig is null)
        throw new FatalUserException("At least 1 workspace is required per monitor.");

      // Assign the workspace to the empty monitor.
      _bus.Invoke(new ActivateWorkspaceCommand(inactiveWorkspaceConfig.Name, monitor));
    }
  }
}
