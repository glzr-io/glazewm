using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  class DetachWorkspaceFromMonitorHandler : ICommandHandler<DetachWorkspaceFromMonitorCommand>
  {
    public Bus _bus { get; }

    public DetachWorkspaceFromMonitorHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(DetachWorkspaceFromMonitorCommand command)
    {
      var workspace = command.Workspace;

      _bus.Invoke(new DetachContainerCommand(workspace));
      _bus.RaiseEvent(new WorkspaceDetachedEvent(command.Workspace));

      return CommandResponse.Ok;
    }
  }
}
