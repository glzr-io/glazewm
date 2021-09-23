using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.CommandHandlers
{
  class AddMonitorHandler : ICommandHandler<AddMonitorCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;

    public AddMonitorHandler(Bus bus, ContainerService containerService, WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(AddMonitorCommand command)
    {
      var newMonitor = new Monitor(command.Screen);
      _containerService.ContainerTree.AddChild(newMonitor);

      ActivateWorkspaceOnMonitor(newMonitor);

      _bus.RaiseEvent(new MonitorAddedEvent(newMonitor));

      return CommandResponse.Ok;
    }

    private void ActivateWorkspaceOnMonitor(Monitor monitor)
    {
      // Get first workspace that is not active.
      var inactiveWorkspace = _workspaceService.InactiveWorkspaces.ElementAtOrDefault(0);

      if (inactiveWorkspace == null)
        throw new FatalUserException("At least 1 workspace is required per monitor.");

      // Assign the workspace to the newly added monitor.
      _bus.Invoke(new AttachWorkspaceToMonitorCommand(inactiveWorkspace, monitor));

      // Display the workspace (since it's the only one on the monitor).
      _bus.Invoke(new DisplayWorkspaceCommand(inactiveWorkspace));
    }
  }
}
