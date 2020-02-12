using LarsWM.Domain.Common.Models;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    class DisplayWorkspaceCommand : Command
    {
        public Guid WorkspaceId { get; private set; }

        public DisplayWorkspaceCommand(Guid workspaceId)
        {
            WorkspaceId = workspaceId;
        }
    }
}
