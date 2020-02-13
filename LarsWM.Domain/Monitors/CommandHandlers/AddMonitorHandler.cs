using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.CommandHandlers
{
    class AddMonitorHandler : ICommandHandler<AddMonitorCommand>
    {
        private IBus _bus;
        private MonitorService _monitorService;

        public AddMonitorHandler(IBus bus, MonitorService monitorService)
        {
            _bus = bus;
            _monitorService = monitorService;
        }

        public CommandResponse Handle(AddMonitorCommand command)
        {
            var newMonitor = new Monitor(command.Screen);
            _monitorService.Monitors.Add(newMonitor);

            _bus.RaiseEvent(new MonitorAddedEvent(newMonitor.Id));

            return new CommandResponse(true, newMonitor.Id);
        }
    }
}
