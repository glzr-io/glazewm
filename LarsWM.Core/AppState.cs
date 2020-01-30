using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM.Core
{
    public class AppState
    {
        // Not sure whether these should be BehaviorSubjects.
        public List<Monitor> Monitors = new List<Monitor>();
        public List<Workspace> Workspaces = new List<Workspace>();
        public List<Window> Windows = new List<Window>();

        // Create method InitialiseState that invokes AddMonitorCommand etc?
    }
}
