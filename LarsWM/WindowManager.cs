using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace LarsWM
{
    enum FocusDirection
    {
        Up,
        Down,
        Left,
        Right,
    }

    class WindowManager
    {
        private List<Monitor> _monitors = new List<Monitor>();

        public WindowManager()
        {
            foreach (var screen in Screen.AllScreens)
            {
                _monitors.Add(new Monitor(screen));
            }

            var focusedMonitor = _monitors.Find(m => m.IsPrimary);

            int index = 0;
            foreach (var monitor in _monitors) 
            { 
                monitor.WorkspacesInMonitor.Add(new Workspace(index, new List<Window>()));
                index++;
            } 
        }

        public int NumMonitors => _monitors.Count;

        public void ShiftFocusInDirection(FocusDirection direction)
        { }

        public void ShiftFocusToWorkspace(Workspace workspace)
        { }

        public void MoveFocusedWindowToWorkspace(Window window, Workspace workspace)
        { } 

        public void MoveFocusedWindowInDirection(Window window, FocusDirection direction)
        { }

    }
}

