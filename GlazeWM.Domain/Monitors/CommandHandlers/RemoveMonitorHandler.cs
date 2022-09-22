using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.CommandHandlers
{
  internal class RemoveMonitorHandler : ICommandHandler<RemoveMonitorCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;

    public RemoveMonitorHandler(Bus bus, MonitorService monitorService)
    {
      _bus = bus;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(RemoveMonitorCommand command)
    {
      var monitorToRemove = command.MonitorToRemove;
      var targetMonitor = _monitorService.GetMonitors().First(
        monitor => monitor != monitorToRemove
      );

      // Keep reference to the focused monitor prior to moving workspaces around.
      var focusedMonitor = _monitorService.GetFocusedMonitor();

      // Avoid moving empty workspaces.
      var workspacesToMove = monitorToRemove.Children
        .Cast<Workspace>()
        .Where(workspace => workspace.HasChildren() || workspace.KeepAlive);

      foreach (var workspace in workspacesToMove.ToList())
      {
        // Move workspace to target monitor.
        _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, targetMonitor, false));

        // Update workspaces displayed in bar window.
        // TODO: Consider creating separate event `WorkspaceMovedEvent`.
        _bus.Emit(new WorkspaceActivatedEvent(workspace));
      }

      _bus.Invoke(new DetachContainerCommand(monitorToRemove));
      _bus.Emit(new MonitorRemovedEvent(monitorToRemove.DeviceName));

      return CommandResponse.Ok;
    }
  }
}
