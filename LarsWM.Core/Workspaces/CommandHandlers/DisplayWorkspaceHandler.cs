using LarsWM.Core.Common.Models;
using LarsWM.Core.Monitors;
using LarsWM.Core.Workspaces.Commands;

namespace LarsWM.Core.Workspaces.CommandHandlers
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

        public void Handle(DisplayWorkspaceCommand command)
        {
            var workspace = _workspaceService.GetWorkspaceById(command.WorkspaceId);
            var parentMonitor = _monitorService.GetMonitorFromWorkspace(workspace);

            parentMonitor.DisplayedWorkspace = workspace;
        }
    }
}
