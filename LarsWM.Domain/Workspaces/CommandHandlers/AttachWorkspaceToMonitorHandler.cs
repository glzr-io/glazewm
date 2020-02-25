using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class AttachWorkspaceToMonitorHandler : ICommandHandler<AttachWorkspaceToMonitorCommand>
    {
        public dynamic Handle(AttachWorkspaceToMonitorCommand command)
        {
            command.Monitor.AddChild(command.Workspace);

            return new CommandResponse(true, command.Monitor.Id);
        }
    }
}
