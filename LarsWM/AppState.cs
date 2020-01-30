using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM
{
    public class AppState
    {
        // Not sure whether to change this to a BehaviorSubject.
        public List<Monitor> Monitors = new List<Monitor>();
        public List<Workspace> Workspaces => Monitors.Select(m => m.WorkspacesInMonitor) as List<Workspace>;
        public List<Window> Windows => Workspaces.Select(w => w.WindowsInWorkspace) as List<Window>;
    }
}
