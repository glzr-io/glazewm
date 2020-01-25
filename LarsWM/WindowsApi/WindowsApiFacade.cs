using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using static LarsWM.WindowsApi.WindowsApiService;

namespace LarsWM.WindowsApi
{
    class WindowsApiFacade
    {
        public static Window[] GetOpenWindows()
        {
            Predicate<Window> OpenWindowsPredicate = (Window window) => {
                var isApplicationWindow = IsWindowVisible(window.Hwnd) && !HasWindowStyle(window.Hwnd, WS.WS_CHILD) && !HasWindowExStyle(window.Hwnd, WS_EX.WS_EX_NOACTIVATE);

                if (!isApplicationWindow)
                {
                    return false;
                }
                return true;
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
    }
}
