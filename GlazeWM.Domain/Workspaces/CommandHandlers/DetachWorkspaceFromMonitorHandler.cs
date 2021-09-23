using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  class DetachWorkspaceFromMonitorHandler : ICommandHandler<DetachWorkspaceFromMonitorCommand>
  {
    private WorkspaceService _workspaceService;
    public Bus _bus { get; }

    public DetachWorkspaceFromMonitorHandler(WorkspaceService workspaceService, Bus bus)
    {
      _workspaceService = workspaceService;
      _bus = bus;
    }

    public CommandResponse Handle(DetachWorkspaceFromMonitorCommand command)
    {
      var monitor = command.Workspace.Parent as Monitor;
      monitor.RemoveChild(command.Workspace);

      _workspaceService.InactiveWorkspaces.Add(command.Workspace);

      _bus.RaiseEvent(new WorkspaceDetachedEvent(command.Workspace));

      return CommandResponse.Ok;
    }
  }
}
