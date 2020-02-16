using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.CommandHandlers
{
    // TODO: Consider moving this to Workspaces domain.
    class AssignWorkspaceToMonitorHandler : ICommandHandler<AssignWorkspaceToMonitorCommand>
    {
        private MonitorService _monitorService { get; }
        private WorkspaceService _workspaceService { get; }

        public AssignWorkspaceToMonitorHandler(MonitorService monitorService, WorkspaceService workspaceService)
        {
            _monitorService = monitorService;
            _workspaceService = workspaceService;
        }

        public CommandResponse Handle(AssignWorkspaceToMonitorCommand command)
        {
            var monitor = _monitorService.GetMonitorById(command.MonitorId);
            var workspace = _workspaceService.GetWorkspaceById(command.WorkspaceId);

            monitor.WorkspacesInMonitor.Add(workspace);

            return new CommandResponse(true, workspace.Id);
        }
    }
}
