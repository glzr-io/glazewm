using LarsWM.Bar.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Infrastructure.Bussing;
using System.Threading;
using System.Windows.Forms;

namespace LarsWM.Bar.CommandHandlers
{
    class LaunchBarOnMonitorHandler : ICommandHandler<LaunchBarOnMonitorCommand>
    {
        private MonitorService _monitorService { get; }
        public IBus _bus { get; }

        public LaunchBarOnMonitorHandler(MonitorService monitorService, IBus bus)
        {
            _monitorService = monitorService;
            _bus = bus;
        }

        public CommandResponse Handle(LaunchBarOnMonitorCommand command)
        {
            var monitor = _monitorService.GetMonitorById(command.MonitorId);

            var thread = new Thread(() =>
            {
                // TODO: Set bar width to width of monitor and launch bar on given monitor.
                var bar = new MainWindow(monitor, _bus);
                bar.Show();
                Application.Run();
            });
            thread.Name = "LarsWMBar";
            thread.SetApartmentState(ApartmentState.STA);
            thread.Start();

            return CommandResponse.Ok;
        }
    }
}
