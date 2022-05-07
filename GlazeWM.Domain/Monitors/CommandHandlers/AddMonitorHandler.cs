using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.CommandHandlers
{
  class AddMonitorHandler : ICommandHandler<AddMonitorCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

    public AddMonitorHandler(Bus bus, ContainerService containerService, WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(AddMonitorCommand command)
    {
      var screen = command.Screen;

      // Create a `Monitor` instance. Use the working area of the monitor instead of the bounds of
      // the display. The working area excludes taskbars and other reserved display space.
      var newMonitor = new Monitor(
        screen.DeviceName,
        screen.WorkingArea.Width,
        screen.WorkingArea.Height,
        screen.WorkingArea.X,
        screen.WorkingArea.Y,
        screen.Primary
      );

      var rootContainer = _containerService.ContainerTree;
      _bus.Invoke(new AttachContainerCommand(newMonitor, rootContainer));

      ActivateWorkspaceOnMonitor(newMonitor);

      _bus.RaiseEvent(new MonitorAddedEvent(newMonitor));

      return CommandResponse.Ok;
    }

    private void ActivateWorkspaceOnMonitor(Monitor monitor)
    {
      // Get name of first workspace that is not active.
      var inactiveWorkspaceName =
        _workspaceService.GetInactiveWorkspaceNames().ElementAtOrDefault(0);

      if (inactiveWorkspaceName == null)
        throw new FatalUserException("At least 1 workspace is required per monitor.");

      // Assign the workspace to the newly added monitor.
      _bus.Invoke(new ActivateWorkspaceCommand(inactiveWorkspaceName, monitor));

      var workspace = _workspaceService.GetActiveWorkspaceByName(inactiveWorkspaceName);

      // Display the workspace (since it's the only one on the monitor).
      _bus.Invoke(new DisplayWorkspaceCommand(workspace));
    }
  }
}
