using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Bar.CommandHandlers
{
    class LaunchBarOnMonitorHandler : ICommandHandler<LaunchBarOnMonitorCommand>
    {
        public void Handle(LaunchBarOnMonitorCommand command)
        {
            var monitor = command.Monitor;

            // TODO: Set bar width to width of monitor and launch bar on given monitor.
            var bar = new MainWindow();
            bar.Show();
        }
    }
}
