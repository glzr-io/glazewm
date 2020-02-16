using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    class CreateWorkspaceCommand : Command
    {
        public string WorkspaceName { get; private set; }

        public CreateWorkspaceCommand(string workspaceName)
        {
            WorkspaceName = workspaceName;
        }
    }
}
