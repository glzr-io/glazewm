using LarsWM.Domain.Common.Models;
using System;

namespace LarsWM.Domain.Workspaces.Commands
{
    class CreateWorkspaceCommand : Command
    {
        public Guid ParentMonitorId { get; private set; }
        public int Index { get; private set; }

        public CreateWorkspaceCommand(Guid parentMonitorId, int index)
        {
            ParentMonitorId = parentMonitorId;
            Index = index;
        }
    }
}
