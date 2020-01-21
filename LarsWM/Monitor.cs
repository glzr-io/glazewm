using System;
using System.Collections.Generic;
using System.Text;
using System.Windows.Forms;

namespace LarsWM
{
    public class Monitor
    {
        public int Index { get; private set; }
        public string Name => Screen.DeviceName;
        public int Width => Screen.WorkingArea.Width;
        public int Height => Screen.WorkingArea.Height;
        public int X => Screen.WorkingArea.X;
        public int Y => Screen.WorkingArea.Y;

        public Screen Screen { get; }

        public Monitor(int index, Screen screen)
        {
            Index = index;
            Screen = screen;
        }
    }
}
