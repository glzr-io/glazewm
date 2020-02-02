using LarsWM.Core.Common.Models;
using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors.Commands;
using LarsWM.Core.Monitors.Events;

namespace LarsWM.Core.Monitors.CommandHandlers
{
    class AddMonitorHandler : ICommandHandler<AddMonitorCommand>
    {
        private AppState _appState;
        private IBus _bus;

        public AddMonitorHandler(IBus bus, AppState appState)
        {
            _bus = bus;
            _appState = appState;
        }

        public void Handle(AddMonitorCommand command)
        {
            _appState.Monitors.Add(new Monitor(command.Screen));

            _bus.RaiseEvent(new MonitorAddedEvent());
        }
    }
}
