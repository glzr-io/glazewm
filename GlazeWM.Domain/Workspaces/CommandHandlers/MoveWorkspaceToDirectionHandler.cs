using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWorkspaceInDirectionHandler : ICommandHandler<MoveWorkspaceInDirectionCommand>
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
      // Get focused workspace
      var workspace = _workspaceService.GetFocusedWorkspace();

      // Get monitor to move focused workspace
      var focusedMonitor = _monitorService.GetFocusedMonitor();
      var targetMonitor = _monitorService.GetMonitorInDirection(command.Direction, focusedMonitor);

      if (targetMonitor != null && targetMonitor != focusedMonitor)
      {
        // Move workspace to target monitor.
        _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, targetMonitor, shouldAdjustSize: false));

        // Update floating placement since the window have to cross monitor
        foreach (var window in workspace.Descendants.OfType<Window>())
        {
          window.FloatingPlacement =
            window.FloatingPlacement.TranslateToCenter(targetMonitor.DisplayedWorkspace.ToRect());
        }

        _bus.Invoke(new RedrawContainersCommand());
      }

      return CommandResponse.Ok;
    }
  }
}
