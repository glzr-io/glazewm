using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Monitors.Commands
{
    class AssignWorkspaceToMonitorCommand : Command
    {
        public Guid WorkspaceId { get; }
        public Guid MonitorId { get; }

        public AssignWorkspaceToMonitorCommand(Guid workspaceId, Guid monitorId)
        {
            WorkspaceId = workspaceId;
            MonitorId = monitorId;
        }
    }
}
