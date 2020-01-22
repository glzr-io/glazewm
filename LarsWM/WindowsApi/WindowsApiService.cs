using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

namespace LarsWM.WindowsApi
{
    class WindowsApiService
    {
        public enum SWP : uint
		{
			SWP_SHOWWINDOW = 0x0040,
			SWP_HIDEWINDOW = 0x0080,
			SWP_NOZORDER = 0x0004,
			SWP_NOREDRAW = 0x0008,
			SWP_NOACTIVATE = 0x0010,
			SWP_NOMOVE = 0x0002,
			SWP_NOSIZE = 0x0001,
			SWP_FRAMECHANGED = 0x0020,
			SWP_NOCOPYBITS = 0x0100,
			SWP_NOOWNERZORDER = 0x0200,
			SWP_DEFERERASE = 0x2000,
			SWP_NOSENDCHANGING = 0x0400,
			SWP_ASYNCWINDOWPOS = 0x4000
		}

        [DllImport("user32.dll")]
        public static extern IntPtr BeginDeferWindowPos(int nNumWindows);

		[DllImport("user32.dll")]
		public static extern IntPtr DeferWindowPos(IntPtr hWinPosInfo, IntPtr hWnd,
			 [Optional] IntPtr hWndInsertAfter, int x, int y, int cx, int cy, SWP uFlags);

		[DllImport("user32.dll")]
		[return: MarshalAs(UnmanagedType.Bool)]
		public static extern bool EndDeferWindowPos(IntPtr hWinPosInfo);

        [DllImport("user32.dll")]
        public static extern IntPtr GetForegroundWindow();

        [DllImport("user32.dll")]
        public static extern bool MoveWindow(IntPtr hWnd, int X, int Y, int nWidth, int nHeight, bool bRepaint);

        [DllImport("user32.dll")]
        public static extern bool SetFocus(IntPtr hWnd);

        [DllImport("user32.dll")]
        public static extern bool ShowWindow(IntPtr hWnd, int X);
    }
}
