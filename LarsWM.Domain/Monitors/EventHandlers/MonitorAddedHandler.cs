using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces.Commands;
using System.Linq;
using LarsWM.Domain.Workspaces;

namespace LarsWM.Domain.Monitors.EventHandler
{
  class MonitorAddedHandler : IEventHandler<MonitorAddedEvent>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;

    public MonitorAddedHandler(Bus bus, WorkspaceService workspaceService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
    }

    // TODO: Move this to `AddMonitorHandler`.
    public void Handle(MonitorAddedEvent @event)
    {
      // Get first workspace that is not active.
      var inactiveWorkspace = _workspaceService.InactiveWorkspaces.ElementAtOrDefault(0);

      if (inactiveWorkspace == null)
        throw new FatalUserException("At least 1 workspace is required per monitor.");

      // Assign the workspace to the newly added monitor.
      _bus.Invoke(new AttachWorkspaceToMonitorCommand(inactiveWorkspace, @event.AddedMonitor));

      // Display the workspace (since it's the only one on the monitor).
      _bus.Invoke(new DisplayWorkspaceCommand(inactiveWorkspace));
    }
  }
}
