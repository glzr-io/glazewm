using LarsWM.Core.Common.Models;
using System;

namespace LarsWM.Core.Workspaces.Commands
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
