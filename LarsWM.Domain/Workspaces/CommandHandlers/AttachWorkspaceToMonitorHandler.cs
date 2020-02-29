using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class AttachWorkspaceToMonitorHandler : ICommandHandler<AttachWorkspaceToMonitorCommand>
    {
        private WorkspaceService _workspaceService;

        public AttachWorkspaceToMonitorHandler(WorkspaceService workspaceService)
        {
            _workspaceService = workspaceService;
        }
        public dynamic Handle(AttachWorkspaceToMonitorCommand command)
        {
            command.Monitor.AddChild(command.Workspace);
            _workspaceService.InactiveWorkspaces.Remove(command.Workspace);

            return new CommandResponse(true, command.Workspace.Id);
        }
    }
}
