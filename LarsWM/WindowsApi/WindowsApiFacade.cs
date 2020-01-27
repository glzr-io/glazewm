using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Runtime.InteropServices;
using static LarsWM.WindowsApi.WindowsApiService;
using static LarsWM.UserConfig.UserConfigService;
using LarsWM.WindowsApi.DataTypes;

namespace LarsWM.WindowsApi
{
    class WindowsApiFacade
    {
        public static Window[] GetOpenWindows()
        {
            Predicate<Window> OpenWindowsPredicate = (Window window) => {
                var isApplicationWindow = IsWindowVisible(window.Hwnd) && !HasWindowStyle(window.Hwnd, WS.WS_CHILD) && !HasWindowExStyle(window.Hwnd, WS_EX.WS_EX_NOACTIVATE);

                var isCurrentProcess = window.Process.Id == Process.GetCurrentProcess().Id;

                var isExcludedClassName = WindowClassesToIgnore.Contains(window.ClassName);
                var isExcludedProcessName = ProcessNamesToIgnore.Contains(window.Process.ProcessName);

                if (isApplicationWindow && !isCurrentProcess && !isExcludedClassName && !isExcludedProcessName)
                {
                    return true;
                }

                return false;
            };

            return FilterToplevelWindows(OpenWindowsPredicate);
        }

        /// <summary>
        /// Returns all top-level windows that match the given predicate.
        /// </summary>
        /// <param name="predicate">The predicate to filter by.</param>
        public static Window[] FilterToplevelWindows(Predicate<Window> predicate)
        {
            List<Window> matchedWindows = new List<Window>();

            EnumWindows(new EnumWindowsDelegate(delegate (IntPtr hwnd, int lParam)
            {
                Window window = new Window(hwnd);

                if (predicate(window))
                {
                    matchedWindows.Add(window);
                }
                return true;
            }), new IntPtr(0));

            return matchedWindows.ToArray();
        }

        /// <summary>
        /// Get the id of the process that created the window.
        /// </summary>
        public static Process GetProcessOfWindow(Window window)
        {
            uint processId;
            GetWindowThreadProcessId(window.Hwnd, out processId);

            try
            {
                return Process.GetProcesses().First(process => process.Id == (int)processId);
            } catch(InvalidOperationException)
            {
                return null;
            }
        }

        /// <summary>
        /// Get the name of the class of the window.
        /// </summary>
        public static string GetClassNameOfWindow(Window window)
        {
            var buffer = new StringBuilder(255);
            GetClassName(window.Hwnd, buffer, buffer.Capacity + 1);
            return buffer.ToString();
        }


        /// <summary>
        /// Get dimensions of the bounding rectangle of the specified window.
        /// </summary>
        public static WindowRect GetLocationOfWindow(Window window)
        {
            WindowRect rect = new WindowRect();
            GetWindowRect(window.Hwnd, ref rect);
            return rect;
        }

        public static bool HasWindowStyle(IntPtr hwnd, WS style)
        {
            var styles = unchecked((WS)GetWindowLongPtr(hwnd, (int)(GWL_STYLE)).ToInt64());

            return (styles & style) != 0;
        }

        public static bool HasWindowExStyle(IntPtr hwnd, WS_EX style)
        {
            var styles = unchecked((WS_EX)GetWindowLongPtr(hwnd, (int)(GWL_EXSTYLE)).ToInt64());

            return (styles & style) != 0;
        }

        private static IntPtr GetWindowLongPtr(IntPtr hWnd, int nIndex)
        {
            if (Environment.Is64BitProcess)
            {
                return GetWindowLongPtr64(hWnd, nIndex);
            }
            else
            {
                return new IntPtr(GetWindowLong32(hWnd, nIndex));
            }
        }

        public static bool IsWindowCloaked(Window window)
        {

            bool isCloaked;
            DwmGetWindowAttribute(window.Hwnd, DwmWindowAttribute.DWMWA_CLOAKED, out isCloaked, Marshal.SizeOf(typeof(bool)));
            return isCloaked;
        }
    }
}
