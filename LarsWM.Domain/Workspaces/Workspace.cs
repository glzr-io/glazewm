using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Windows;
using System;
using System.Collections.Generic;

namespace LarsWM.Domain.Workspaces
{
    public class Workspace : SplitContainer
    {
        public Guid Id = Guid.NewGuid();
        public string Name { get; set; }
        public Window LastFocusedWindow { get; set; }
        public List<Window> WindowsInWorkspace { get; set; } = new List<Window>();
        public int Width => this.Parent.Width - (20 / 2);
        public int Height => this.Parent.Height - (30 / 2);

        public Workspace(string name)
        {
            Name = name;
        }
    }
}
