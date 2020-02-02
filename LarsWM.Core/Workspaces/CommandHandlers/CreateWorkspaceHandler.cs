using LarsWM.Core.Common.Models;
using LarsWM.Core.Workspaces.Commands;

namespace LarsWM.Core.Workspaces.CommandHandlers
{
    class CreateWorkspaceHandler : ICommandHandler<CreateWorkspaceCommand>
    {
        private AppState _appState;

        public CreateWorkspaceHandler(AppState appState)
        {
            _appState = appState;
        }

        public void Handle(CreateWorkspaceCommand command)
        {
            var newWorkspace = new Workspace(command.Id);
            _appState.Workspaces.Add(newWorkspace);

            var workspaceMonitor = _appState.Monitors.Find(monitor => monitor.Name == command.MonitorName);
            workspaceMonitor.WorkspacesInMonitor.Add(newWorkspace);
        }
    }
}
