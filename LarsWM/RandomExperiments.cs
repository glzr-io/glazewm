using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace LarsWM
{
    class RandomExperiments
    {
        
        static int SW_MAXIMIZE = 3;

        static void Main(string[] args)
        {
            Console.WriteLine("Hello World!");
            var a = GetForegroundWindow();
            Console.WriteLine("aaaaa" + a);
            MoveWindow(a, 40, 0, 500, 600, true);

            foreach (Process proc in Process.GetProcesses())
            {

                Console.WriteLine(proc);
                if (proc.MainWindowTitle.Contains("Spotify"))
                {
                    ShowWindow(proc.MainWindowHandle, SW_MAXIMIZE);
                    SetFocus(proc.MainWindowHandle);
                }
            }
        }

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
