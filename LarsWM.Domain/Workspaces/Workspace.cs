using System;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Windows;

namespace LarsWM.Domain.Workspaces
{
    public class Workspace : SplitContainer
    {
        public Guid Id = Guid.NewGuid();
        public string Name { get; set; }
        public Window LastFocusedWindow { get; set; }
        public List<Window> WindowsInWorkspace { get; set; } = new List<Window>();
        private static int OuterGap = 20;
        public override int Height => Parent.Height - OuterGap;
        public override int Width => Parent.Width - OuterGap;
        public override int X => Parent.X + (OuterGap / 2);
        public override int Y => Parent.Y + (OuterGap / 2);

        public Workspace(string name)
        {
            Name = name;
        }
    }
}
