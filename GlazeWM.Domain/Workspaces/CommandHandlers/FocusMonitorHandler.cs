using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal sealed class FocusMonitorHandler : ICommandHandler<FocusMonitorCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;

    public FocusMonitorHandler(
      Bus bus,
      ContainerService containerService,
      MonitorService monitorService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(FocusMonitorCommand command)
    {
      var direction = command.Direction;
      var focusedContainer = _containerService.FocusedContainer;
      var focusedMonitor = _monitorService.GetFocusedMonitor();
      var monitorInDirection = _monitorService.GetMonitorInDirection(direction, focusedMonitor);
      var focusTarget = monitorInDirection?.DisplayedWorkspace;

      if (focusTarget is null || focusTarget == focusedContainer)
        return CommandResponse.Ok;

      _bus.Invoke(new SetFocusedDescendantCommand(focusTarget));
      _containerService.HasPendingFocusSync = true;
      _workspaceService.MostRecentWorkspace = focusedMonitor.DisplayedWorkspace;

      return CommandResponse.Ok;
    }
  }
}
