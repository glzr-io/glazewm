using LarsWM.Core.Monitors;
using LarsWM.Core.UserConfigs;
using LarsWM.Core.Windows;
using LarsWM.Core.Workspaces;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM.Core
{
    public class AppState
    {
        // Not sure whether these should be BehaviorSubjects.
        public List<Monitor> Monitors { get; set; } = new List<Monitor>();
        public List<Workspace> Workspaces { get; set; } = new List<Workspace>();
        public List<Window> Windows { get; set; } = new List<Window>();

        // TODO: Create method InitialiseState that invokes AddMonitorCommand etc?
    }
}
