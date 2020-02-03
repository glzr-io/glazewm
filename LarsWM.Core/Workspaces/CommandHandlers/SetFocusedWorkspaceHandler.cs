using LarsWM.Core.Common.Models;
using LarsWM.Core.Monitors;
using LarsWM.Core.Workspaces.Commands;

namespace LarsWM.Core.Workspaces.CommandHandlers
{
    class SetFocusedWorkspaceHandler : ICommandHandler<SetFocusedWorkspaceCommand>
    {
        private AppState _appState;
        private MonitorService _monitorService;

        public SetFocusedWorkspaceHandler(AppState appState, MonitorService monitorService)
        {
            _appState = appState;
            _monitorService = monitorService;
        }

        public void Handle(SetFocusedWorkspaceCommand command)
        {
            throw new System.NotImplementedException();

            var workspace = _workspaceService.GetWorkspaceById(command.WorkspaceId);
            var parentMonitor = _monitorService.GetMonitorOfWorkspace(workspace);
            parentMonitor.DisplayedWorkspace = workspace;

            if (workspace.WindowsInWorkspace.Count() > 0)
            {
                // TODO: Set focus through windows api
            }
        }
    }
}
