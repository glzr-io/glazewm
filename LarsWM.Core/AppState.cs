using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors;
using LarsWM.Core.Monitors.Commands;
using LarsWM.Core.UserConfigs;
using LarsWM.Core.UserConfigs.Commands;
using LarsWM.Core.Windows;
using LarsWM.Core.Workspaces;
using static LarsWM.Core.WindowsApi.WindowsApiFacade;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace LarsWM.Core
{
    public class AppState
    {
        // TODO: Change List types to BehaviorSubjects?
        public List<Monitor> Monitors { get; set; } = new List<Monitor>();
        public List<Workspace> Workspaces { get; set; } = new List<Workspace>();
        public List<Window> Windows { get; set; } = new List<Window>();
        public UserConfig UserConfig { get; set; } = null;

        private IBus _bus;
        private MonitorService _monitorService;

        public AppState(IBus bus, MonitorService monitorService)
        {
            _bus = bus;
            _monitorService = monitorService;
        }

        public void InitialiseState()
        {
            // Read user config file and set its values in state.
            // TODO: Rename to ReadUserConfigCommand
            _bus.Invoke(new GetUserConfigCommand());

            // Create a Monitor and consequently a Workspace for each detected Screen.
            foreach (var screen in Screen.AllScreens)
            {
                _bus.Invoke(new AddMonitorCommand(screen));
            }

            // TODO: move the below code to its own command
            var windows = GetOpenWindows();

            foreach (var window in windows)
            {
                // Add window to its nearest workspace
                var targetMonitor = _monitorService.GetMonitorFromWindowHandle(window);
                targetMonitor.DisplayedWorkspace.WindowsInWorkspace.Add(window);
            }
        }
    }
}
