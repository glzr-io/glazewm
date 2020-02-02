using LarsWM.Core.Monitors;
using LarsWM.Core.UserConfigs;
using LarsWM.Core.Windows;
using LarsWM.Core.Workspaces;
using System.Collections.Generic;

namespace LarsWM.Core
{
    public class AppState
    {
        // TODO: Change List types to BehaviorSubjects?
        public List<Monitor> Monitors { get; set; } = new List<Monitor>();
        public List<Workspace> Workspaces { get; set; } = new List<Workspace>();
        public List<Window> Windows { get; set; } = new List<Window>();
        public UserConfig UserConfig { get; set; } = null;
    }
}
