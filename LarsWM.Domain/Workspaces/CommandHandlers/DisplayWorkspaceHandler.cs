using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class DisplayWorkspaceHandler : ICommandHandler<DisplayWorkspaceCommand>
    {
        private IBus _bus;
        private MonitorService _monitorService;

        public DisplayWorkspaceHandler(IBus bus, MonitorService monitorService)
        {
            _bus = bus;
            _monitorService = monitorService;
        }

        public dynamic Handle(DisplayWorkspaceCommand command)
        {
            var workspace = command.Workspace;

            var monitor = _monitorService.GetMonitorFromChildContainer(command.Workspace);
            monitor.DisplayedWorkspace = command.Workspace;

            return new CommandResponse(true, command.Workspace.Id);
        }
    }
}
