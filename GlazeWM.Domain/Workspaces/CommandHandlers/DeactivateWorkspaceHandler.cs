using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class DeactivateWorkspaceHandler : ICommandHandler<DeactivateWorkspaceCommand>
  {
    public Bus _bus { get; }

    public DeactivateWorkspaceHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(DeactivateWorkspaceCommand command)
    {
      var workspace = command.Workspace;

      _bus.Invoke(new DetachContainerCommand(workspace));
      _bus.RaiseEvent(new WorkspaceDeactivatedEvent(workspace));

      return CommandResponse.Ok;
    }
  }
}
