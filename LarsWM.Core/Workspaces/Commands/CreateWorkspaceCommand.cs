using LarsWM.Core.Common.Models;
using System;

namespace LarsWM.Core.Workspaces.Commands
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
