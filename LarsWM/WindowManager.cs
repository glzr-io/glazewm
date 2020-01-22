using System;
using System.Collections.Generic;
using System.Diagnostics;
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
        private Monitor[] _monitors;

        public WindowManager()
        {
            var screens = Screen.AllScreens;
            _monitors = new Monitor[screens.Length];

            _monitors[0] = new Monitor(0, Screen.PrimaryScreen);

            var index = 1;
            foreach (var screen in screens)
            {
                if (!screen.Primary)
                {
                    _monitors[index] = new Monitor(index, screen);
                    index++;
                }
            }

            var focusedMonitor = _monitors[0];
        }

        public int NumMonitors => _monitors.Length;

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

