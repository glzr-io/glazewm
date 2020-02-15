using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Bar.Commands
{
    class LaunchBarOnMonitorCommand : Command
    {
        public Guid MonitorId { get; }

        public LaunchBarOnMonitorCommand(Guid monitorId)
        {
            MonitorId = monitorId;
        }
    }
}
