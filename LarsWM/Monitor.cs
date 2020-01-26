using System;
using System.Collections.Generic;
using System.Text;
using System.Windows.Forms;

namespace LarsWM
{
    public class Monitor
    {
        public List<Workspace> WorkspacesInMonitor = new List<Workspace>();
        public Workspace DisplayedWorkspace;  // Alternatively add IsDisplayed/IsVisible property to Workspace instance
        public string Name => Screen.DeviceName;
        public int Width => Screen.WorkingArea.Width;
        public int Height => Screen.WorkingArea.Height;
        public int X => Screen.WorkingArea.X;
        public int Y => Screen.WorkingArea.Y;
        public bool IsPrimary => Screen.Primary;

        public Screen Screen { get; }

        public Monitor(Screen screen)
        {
            Screen = screen;
        }
    }
}
