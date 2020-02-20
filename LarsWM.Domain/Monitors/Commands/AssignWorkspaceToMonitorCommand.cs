using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.Commands
{
    class AssignWorkspaceToMonitorCommand : Command
    {
        public Workspace Workspace { get; }
        public Monitor Monitor { get; }

        public AssignWorkspaceToMonitorCommand(Workspace workspace, Monitor monitor)
        {
            Workspace = workspace;
            Monitor = monitor;
        }
    }
}
