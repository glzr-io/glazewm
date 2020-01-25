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
                if (IsAppWindow(window.Hwnd))
                {
                    return true;
                }
                return false;
            };

            return FilterToplevelWindows(OpenWindowsPredicate);
        }

        /// <summary>
        /// Returns all toplevel windows that match the given predicate.
        /// </summary>
        /// <param name="predicate">The predicate to filter.</param>
        /// <returns>The filtered toplevel windows</returns>
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

        public static bool IsAppWindow(IntPtr hwnd)
        {
            return IsWindowVisible(hwnd) && !HasWindowStyle(hwnd, WS.WS_CHILD) && !HasWindowExStyle(hwnd, WS_EX.WS_EX_NOACTIVATE);
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
            if (IntPtr.Size == 8)
                return GetWindowLongPtr64(hWnd, nIndex);
            else
                return new IntPtr(GetWindowLong32(hWnd, nIndex));
        }

        public static WS_EX GetWindowExStyleLongPtr(IntPtr hwnd)
        {
            if (Environment.Is64BitProcess)
            {
                return (WS_EX)GetWindowLongPtr(hwnd, GWL_EXSTYLE);
            }
            else
            {
                return (WS_EX)GetWindowLong(hwnd, GWL_EXSTYLE);
            }
        }

        //public static WS GetWindowStyleLongPtr(IntPtr hwnd)
        //{
        //    if (Environment.Is64BitProcess)
        //    {
        //        return (WS)GetWindowLongPtr(hwnd, GWL_STYLE);
        //    }
        //    else
        //    {
        //        return (WS)GetWindowLong(hwnd, GWL_STYLE);
        //    }
        //}
    }
}
