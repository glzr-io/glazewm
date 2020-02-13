using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces.Commands;
using System.Linq;

namespace LarsWM.Domain.Monitors.EventHandler
{
    class MonitorAddedHandler : IEventHandler<MonitorAddedEvent>
    {
        private IBus _bus;

        public MonitorAddedHandler(IBus bus)
        {
            _bus = bus;
        }

        public void Handle(MonitorAddedEvent @event)
        {
            // Create an initial workspace for the monitor if one doesn't exist.
            // TODO: Replace hardcoded Workspace.Index property.
            var result = _bus.Invoke(new CreateWorkspaceCommand(@event.AddedMonitorId, 1));

            // Set the displayed workspace to the newly created one.
            _bus.Invoke(new DisplayWorkspaceCommand(result.AggregateId));
        }
    }
}
