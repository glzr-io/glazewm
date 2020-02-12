using LarsWM.Domain.Common.Models;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    class SetFocusedWorkspaceCommand : Command
    {
        public Guid WorkspaceId { get; private set; }

        public SetFocusedWorkspaceCommand(Guid workspaceId)
        {
            WorkspaceId = workspaceId;
        }
    }
}
