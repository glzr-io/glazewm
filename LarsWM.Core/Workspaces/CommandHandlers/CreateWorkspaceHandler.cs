using LarsWM.Core.Common.Models;
using LarsWM.Core.Monitors;
using LarsWM.Core.Workspaces.Commands;

namespace LarsWM.Core.Workspaces.CommandHandlers
{
    class CreateWorkspaceHandler : ICommandHandler<CreateWorkspaceCommand>
    {
        private WorkspaceService _workspaceService;
        private MonitorService _monitorService;

        public CreateWorkspaceHandler(WorkspaceService workspaceService, MonitorService monitorService)
        {
            _workspaceService = workspaceService;
            _monitorService = monitorService;
        }

        public void Handle(CreateWorkspaceCommand command)
        {
            var newWorkspace = new Workspace(command.Index);
            _workspaceService.Workspaces.Add(newWorkspace);

            var parentMonitor = _monitorService.GetMonitorById(command.ParentMonitorId);
            parentMonitor.WorkspacesInMonitor.Add(newWorkspace);
        }
    }
}
