using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    class CreateWorkspaceCommand : Command
    {
        public Guid ParentMonitorId { get; private set; }
        public string WorkspaceName { get; private set; }

        public CreateWorkspaceCommand(Guid parentMonitorId, string workspaceName)
        {
            ParentMonitorId = parentMonitorId;
            WorkspaceName = workspaceName;
        }
    }
}
