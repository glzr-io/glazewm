using LarsWM.Core.Common.Models;
using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors.Commands;
using LarsWM.Core.Monitors.Events;

namespace LarsWM.Core.Monitors.CommandHandlers
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

        public void Handle(AddMonitorCommand command)
        {
            _monitorService.Monitors.Add(new Monitor(command.Screen));

            _bus.RaiseEvent(new MonitorAddedEvent());
        }
    }
}
