using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    public class SetFocusedWorkspaceCommand : Command
    {
        public Guid WorkspaceId { get; private set; }

        public SetFocusedWorkspaceCommand(Guid workspaceId)
        {
            WorkspaceId = workspaceId;
        }
    }
}
