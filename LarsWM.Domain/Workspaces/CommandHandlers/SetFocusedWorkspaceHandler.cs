using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class SetFocusedWorkspaceHandler : ICommandHandler<SetFocusedWorkspaceCommand>
    {
        private WorkspaceService _workspaceService;
        private MonitorService _monitorService;

        public SetFocusedWorkspaceHandler(WorkspaceService workspaceService, MonitorService monitorService)
        {
            _workspaceService = workspaceService;
            _monitorService = monitorService;
        }

        public CommandResponse Handle(SetFocusedWorkspaceCommand command)
        {
            throw new System.NotImplementedException();

            //var workspace = _workspaceService.GetWorkspaceById(command.WorkspaceId);
            //var parentMonitor = _monitorService.GetMonitorOfWorkspace(workspace);
            //parentMonitor.DisplayedWorkspace = workspace;

            //if (workspace.WindowsInWorkspace.Count() > 0)
            //{
            //    // TODO: Set focus through windows api
            //}
        }
    }
}
