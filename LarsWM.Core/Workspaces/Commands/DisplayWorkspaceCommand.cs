using LarsWM.Core.Common.Models;
using System;

namespace LarsWM.Core.Workspaces.Commands
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
