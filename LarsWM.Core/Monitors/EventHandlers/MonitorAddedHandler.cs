using LarsWM.Core.Common.Models;
using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors.Events;
using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Monitors.EventHandler
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
            throw new NotImplementedException();

            // Create an initial Workspace for each Monitor
        //    int index = 0;
        //    foreach (var monitor in msg.monitors)
        //    {
        //        // TODO: add IsFocused property to focused window, workspace & monitor
        //        var newWorkspace = new Workspace(index, new List<Window>());
        //        monitor.WorkspacesInMonitor.Add(newWorkspace);
        //        monitor.DisplayedWorkspace = newWorkspace;

        //        index++;
        //    }
        }
    }
}
