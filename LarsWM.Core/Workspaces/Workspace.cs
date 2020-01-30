using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core
{
    public class Workspace
    {
        public int Identifier { get; set; }
        public Window LastFocusedWindow { get; set; }
        public List<Window> WindowsInWorkspace = new List<Window>();

        public Workspace(int identifier, List<Window> windowsInWorkspace) {
            Identifier = identifier;
            WindowsInWorkspace = windowsInWorkspace;
        }
    }
}
