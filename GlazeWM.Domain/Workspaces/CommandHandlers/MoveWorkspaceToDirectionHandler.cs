using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class MoveWorkspaceInDirectionHandler :
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

      // Update floating placement since the window has to cross monitors.
      foreach (var window in focusedWorkspace.Descendants.OfType<Window>())
      {
        window.FloatingPlacement = window.FloatingPlacement.TranslateToCenter(
          targetMonitor.DisplayedWorkspace.ToRect()
        );
      }

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
