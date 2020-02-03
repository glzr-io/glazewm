using LarsWM.Core.Common.Models;
using LarsWM.Core.Monitors;
using LarsWM.Core.Workspaces.Commands;

namespace LarsWM.Core.Workspaces.CommandHandlers
{
    class CreateWorkspaceHandler : ICommandHandler<CreateWorkspaceCommand>
    {
        private AppState _appState;
        private MonitorService _monitorService;

        public CreateWorkspaceHandler(AppState appState, MonitorService monitorService)
        {
            _appState = appState;
            _monitorService = monitorService;
        }

        public void Handle(CreateWorkspaceCommand command)
        {
            var newWorkspace = new Workspace(command.Index);
            _appState.Workspaces.Add(newWorkspace);

            var parentMonitor = _monitorService.GetMonitorById(command.ParentMonitorId);
            parentMonitor.WorkspacesInMonitor.Add(newWorkspace);
        }
    }
}
