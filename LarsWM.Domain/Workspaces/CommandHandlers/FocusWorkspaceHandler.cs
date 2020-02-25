using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class FocusWorkspaceHandler : ICommandHandler<FocusWorkspaceCommand>
    {
        private IBus _bus;
        private WorkspaceService _workspaceService;
        private MonitorService _monitorService;

        public FocusWorkspaceHandler(IBus bus, WorkspaceService workspaceService, MonitorService monitorService)
        {
            _bus = bus;
            _workspaceService = workspaceService;
            _monitorService = monitorService;
        }

        public dynamic Handle(FocusWorkspaceCommand command)
        {
            var workspace = _workspaceService.GetWorkspaceByName(command.WorkspaceName);

            _bus.Invoke(new DisplayWorkspaceCommand(workspace));

            // TODO: Set focus to the last focused window on workspace.

            return CommandResponse.Ok;
        }
    }
}
