using System;
using System.Collections.Generic;
using System.Text;
using static LarsWM.WindowsApi.WindowsApiService;

namespace LarsWM.WindowsApi
{
    class WindowsApiFacade
    {
        public void GetOpenWindows()
        {
            EnumWindows((handle, param) =>
            {
                if (IsAppWindow(handle))
                {
                    Console.WriteLine(handle);
                }
                return true;
            }, IntPtr.Zero);
        }

        public bool IsAppWindow(IntPtr hwnd)
        {
            return IsWindowVisible(hwnd) &&
               !GetWindowExStyleLongPtr(hwnd).HasFlag(WS_EX.WS_EX_NOACTIVATE) &&
               !GetWindowStyleLongPtr(hwnd).HasFlag(WS.WS_CHILD);
        }

        public WS_EX GetWindowExStyleLongPtr(IntPtr hwnd)
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

        public WS GetWindowStyleLongPtr(IntPtr hwnd)
        {
            if (Environment.Is64BitProcess)
            {
                return (WS)GetWindowLongPtr(hwnd, GWL_STYLE);
            }
            else
            {
                return (WS)GetWindowLong(hwnd, GWL_STYLE);
            }
        }
    }
}
