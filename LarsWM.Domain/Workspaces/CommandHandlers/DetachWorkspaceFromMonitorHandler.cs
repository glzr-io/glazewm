using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
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

    public dynamic Handle(DetachWorkspaceFromMonitorCommand command)
    {
      var monitor = command.Workspace.Parent as Monitor;
      monitor.RemoveChild(command.Workspace);

      _workspaceService.InactiveWorkspaces.Add(command.Workspace);

      _bus.RaiseEvent(new WorkspaceDetachedEvent(command.Workspace));

      return new CommandResponse(true, command.Workspace.Id);
    }
  }
}
