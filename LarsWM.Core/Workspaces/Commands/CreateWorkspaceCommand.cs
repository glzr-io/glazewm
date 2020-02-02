using LarsWM.Core.Common.Models;
using LarsWM.Core.Monitors;

namespace LarsWM.Core.Workspaces.Commands
{
    class CreateWorkspaceCommand : Command
    {
        public string MonitorName { get; private set; }
        public int Id { get; private set; }

        public CreateWorkspaceCommand(string monitorName, int id)
        {
            MonitorName = monitorName;
            Id = id;
        }
    }
}
