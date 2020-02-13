using LarsWM.Bar.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Bar.CommandHandlers
{
    class LaunchBarOnMonitorHandler : ICommandHandler<LaunchBarOnMonitorCommand>
    {
        public CommandResponse Handle(LaunchBarOnMonitorCommand command)
        {
            //var monitor = command.Monitor;

            // TODO: Set bar width to width of monitor and launch bar on given monitor.
            var bar = new MainWindow();
            bar.Show();

            return CommandResponse.Ok;
        }
    }
}
