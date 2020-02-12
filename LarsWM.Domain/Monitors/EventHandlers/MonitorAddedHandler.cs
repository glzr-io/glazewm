using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces.Commands;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM.Domain.Monitors.EventHandler
{
    class MonitorAddedHandler : IEventHandler<MonitorAddedEvent>
    {
        private IBus _bus;
        private MonitorService _monitorService;

        public MonitorAddedHandler(IBus bus, MonitorService monitorService)
        {
            _bus = bus;
            _monitorService = monitorService;
        }

        public void Handle(MonitorAddedEvent @event)
        {
            foreach (var monitor in _monitorService.Monitors)
            {
                // Create an initial workspace for the monitor if one doesn't exist.
                if (monitor.WorkspacesInMonitor.Count() == 0)
                    _bus.Invoke(new CreateWorkspaceCommand(monitor.Id, 1));
                    // TODO: invoke SetDisplayedWorkspaceCommand
                    //_bus.Invoke(new DisplayWorkspaceCommand(workspaceId));
            }

        // Old code:
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
