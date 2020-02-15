using LarsWM.Domain.Windows;
using System;
using System.Collections.Generic;

namespace LarsWM.Domain.Workspaces
{
    public class Workspace
    {
        public Guid Id = Guid.NewGuid();
        public string Name { get; set; }
        public Window LastFocusedWindow { get; set; }
        public List<Window> WindowsInWorkspace { get; set; } = new List<Window>();

        public Workspace(string name) {
            Name = name;
        }
    }
}
