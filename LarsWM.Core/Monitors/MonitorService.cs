using LarsWM.Core.Windows;
using LarsWM.Core.Workspaces;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace LarsWM.Core.Monitors
{
    public class MonitorService
    {
        public List<Monitor> Monitors { get; set; } = new List<Monitor>();

        public Monitor GetMonitorById(Guid id)
        {
            return Monitors.FirstOrDefault(m => m.Id == id);
        }

        public Monitor GetMonitorFromWindowHandle(Window window)
        {
            var screen = Screen.FromHandle(window.Hwnd);

            var matchedMonitor = Monitors.FirstOrDefault(m => m.Screen.DeviceName == screen.DeviceName);
            if (matchedMonitor == null)
                return Monitors[0];

            return matchedMonitor;
        }

        public Monitor GetMonitorFromWorkspace(Workspace workspace)
        {
            return Monitors.FirstOrDefault(m => m.WorkspacesInMonitor.Contains(workspace));
        }
    }
}
