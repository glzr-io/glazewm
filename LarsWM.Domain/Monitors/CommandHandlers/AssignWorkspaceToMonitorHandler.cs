using LarsWM.Domain.Monitors.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.CommandHandlers
{
    // TODO: Consider moving this to Workspaces domain.
    class AssignWorkspaceToMonitorHandler : ICommandHandler<AssignWorkspaceToMonitorCommand>
    {
        public dynamic Handle(AssignWorkspaceToMonitorCommand command)
        {
            command.Monitor.AddChild(command.Workspace);

            return new CommandResponse(true, command.Monitor.Id);
        }
    }
}
