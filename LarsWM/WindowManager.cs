using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Windows.Forms;
using static LarsWM.WindowsApi.WindowsApiService;
using static LarsWM.WindowsApi.WindowsApiFacade;

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
            // Create a Monitor instance for each Screen
            foreach (var screen in Screen.AllScreens)
            {
                _monitors.Add(new Monitor(screen));
            }

            var focusedMonitor = _monitors.Find(m => m.IsPrimary);

            // Create an initial Workspace for each Monitor
            int index = 0;
            foreach (var monitor in _monitors)
            {
                // TODO: add IsFocused property to focused window, workspace & monitor
                var newWorkspace = new Workspace(index, new List<Window>());
                monitor.WorkspacesInMonitor.Add(newWorkspace);
                monitor.DisplayedWorkspace = newWorkspace;

                index++;
            }

            var windows = GetOpenWindows();

            foreach (var window in windows)
            {
                DumpManagedWindows(window);

                var targetMonitor = GetMonitorFromWindowHandle(window);
                targetMonitor.DisplayedWorkspace.WindowsInWorkspace.Add(window);
            }

            Debug.WriteLine(_monitors);
        }

        private static void DumpManagedWindows(Window window)
        {
            StringBuilder sb = new StringBuilder(GetWindowTextLength(window.Hwnd) + 1);
            GetWindowText(window.Hwnd, sb, sb.Capacity);
            Debug.WriteLine(sb.ToString());

            uint processId;
            GetWindowThreadProcessId(window.Hwnd, out processId);
            var _processId = (int)processId;
            //Debug.WriteLine(_processId);

            var process = Process.GetProcesses().FirstOrDefault(p => p.Id == _processId);
            var _processName = process.ProcessName;
            Debug.WriteLine(_processName);
            Debug.WriteLine(window.Location);
        }

        public Monitor GetMonitorFromWindowHandle(Window window)
        {
            var screen = Screen.FromHandle(window.Hwnd);
            return _monitors.FirstOrDefault(m => m.Screen.DeviceName == screen.DeviceName) ?? _monitors[0];
        }

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

