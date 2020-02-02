using LarsWM.Core.Windows;
using System.Collections.Generic;

namespace LarsWM.Core.Workspaces
{
    public class Workspace
    {
        public int Id { get; set; }
        public Window LastFocusedWindow { get; set; }
        public List<Window> WindowsInWorkspace { get; set; } = new List<Window>();

        public Workspace(int id) {
            Id = id;
        }
    }
}
