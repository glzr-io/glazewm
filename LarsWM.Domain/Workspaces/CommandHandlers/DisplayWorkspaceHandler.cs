using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class DisplayWorkspaceHandler : ICommandHandler<DisplayWorkspaceCommand>
    {
        private WorkspaceService _workspaceService;
        private MonitorService _monitorService;

        public DisplayWorkspaceHandler(WorkspaceService workspaceService, MonitorService monitorService)
        {
            _workspaceService = workspaceService;
            _monitorService = monitorService;
        }

        public CommandResponse Handle(DisplayWorkspaceCommand command)
        {
            var workspace = _workspaceService.GetWorkspaceById(command.WorkspaceId);
            var parentMonitor = _monitorService.GetMonitorFromWorkspace(workspace);

            parentMonitor.DisplayedWorkspace = workspace;

            return new CommandResponse(true, workspace.Id);
        }
    }
}
