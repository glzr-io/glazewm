using LarsWM.Domain.Containers;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class DisplayWorkspaceHandler : ICommandHandler<DisplayWorkspaceCommand>
    {
        private ContainerService _containerService;

        public DisplayWorkspaceHandler(ContainerService containerService)
        {
            _containerService = containerService;
        }

        public dynamic Handle(DisplayWorkspaceCommand command)
        {
            var monitor = _containerService.GetMonitorForContainer(command.Workspace);

            monitor.DisplayedWorkspace = command.Workspace;

            return new CommandResponse(true, command.Workspace.Id);
        }
    }
}
