using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
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
