using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWorkspaceToMonitorHandler : ICommandHandler<MoveWorkspaceToMonitorCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;

    public MoveWorkspaceToMonitorHandler (
      Bus bus,
      MonitorService monitorService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle (MoveWorkspaceToMonitorCommand command)
    {
      // Get focused workspace
      var workspace = _workspaceService.GetFocusedWorkspace();

      // Get monitor to move focused workspace
      var focusedMonitor = _monitorService.GetFocusedMonitor();
      var targetMonitor = _monitorService.GetMonitorInDirection(command.Direction, focusedMonitor);

      if (targetMonitor != null && targetMonitor != focusedMonitor)
      {
        // Move workspace to target monitor.
        _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, targetMonitor, shouldAdjustSize: false));
      }
            
      return CommandResponse.Ok;
    }
  }
}
