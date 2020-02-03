using LarsWM.Core.Windows;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace LarsWM.Core.Monitors
{
    public class MonitorService
    {
        private AppState _appState;

        public MonitorService(AppState appState)
        {
            _appState = appState;
        }

        public Monitor GetMonitorById(Guid id)
        {
            return _appState.Monitors.FirstOrDefault(m => m.Id == id);
        }

        public Monitor GetMonitorFromWindowHandle(Window window)
        {
            var screen = Screen.FromHandle(window.Hwnd);

            var matchedMonitor = _appState.Monitors.FirstOrDefault(m => m.Screen.DeviceName == screen.DeviceName);
            if (matchedMonitor == null)
                return _appState.Monitors[0];

            return matchedMonitor;
        }
    }
}
