using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces.Events;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class ActivateWorkspaceHandler : ICommandHandler<ActivateWorkspaceCommand>
  {
    private readonly Bus _bus;

    public ActivateWorkspaceHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(ActivateWorkspaceCommand command)
    {
      var workspaceName = command.WorkspaceName;
      var targetMonitor = command.TargetMonitor;

      var newWorkspace = new Workspace(workspaceName);

      // Attach the created workspace to the specified monitor.
      _bus.Invoke(new AttachContainerCommand(newWorkspace, targetMonitor));
      _bus.RaiseEvent(new WorkspaceActivatedEvent(newWorkspace));

      return CommandResponse.Ok;
    }
  }
}
