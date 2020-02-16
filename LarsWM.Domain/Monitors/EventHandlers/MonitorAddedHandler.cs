using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces.Commands;
using System.Linq;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.Commands;

namespace LarsWM.Domain.Monitors.EventHandler
{
    class MonitorAddedHandler : IEventHandler<MonitorAddedEvent>
    {
        private IBus _bus;
        private WorkspaceService _workspaceService;

        public MonitorAddedHandler(IBus bus, WorkspaceService workspaceService)
        {
            _bus = bus;
            _workspaceService = workspaceService;
        }


        public void Handle(MonitorAddedEvent @event)
        {
            var activeWorkspaces = _workspaceService.GetActiveWorkspaces();

            // Get first workspace that is not active.
            var inactiveWorkspace = _workspaceService.Workspaces.FirstOrDefault(w => !activeWorkspaces.Contains(w));

            // Assign the workspace to the newly added monitor.
            var result = _bus.Invoke(new AssignWorkspaceToMonitorCommand(inactiveWorkspace.Id, @event.AddedMonitorId));

            // Display the workspace (since it's the only one on the monitor).
            _bus.Invoke(new DisplayWorkspaceCommand(result.AggregateId));
        }
    }
}
