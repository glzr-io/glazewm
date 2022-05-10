using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class DisplayWorkspaceHandler : ICommandHandler<DisplayWorkspaceCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public DisplayWorkspaceHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DisplayWorkspaceCommand command)
    {
      var workspaceToDisplay = command.Workspace;

      var monitor = MonitorService.GetMonitorFromChildContainer(command.Workspace);
      var currentWorkspace = monitor.DisplayedWorkspace;

      // If `DisplayedWorkspace` is unassigned (ie. on startup), there is no need to show/hide
      // any windows.
      if (currentWorkspace == null)
      {
        monitor.DisplayedWorkspace = workspaceToDisplay;
        return CommandResponse.Ok;
      }

      if (currentWorkspace == workspaceToDisplay)
        return CommandResponse.Ok;

      monitor.DisplayedWorkspace = command.Workspace;

      _containerService.ContainersToRedraw.Add(currentWorkspace);
      _containerService.ContainersToRedraw.Add(workspaceToDisplay);

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
