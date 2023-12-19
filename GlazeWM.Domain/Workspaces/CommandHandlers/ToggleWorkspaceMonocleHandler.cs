using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal sealed class ToggleWorkspaceMonocleHandler : ICommandHandler<ToggleWorkspaceMonocleCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

    public ToggleWorkspaceMonocleHandler(
      Bus bus,
      ContainerService ContainerService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = ContainerService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(ToggleWorkspaceMonocleCommand command)
    {
      var currentWorkspace = _workspaceService.GetFocusedWorkspace();

      currentWorkspace.isMonocle = !currentWorkspace.isMonocle;

      if (currentWorkspace.isMonocle)
        _bus.Emit(new EnterWorkspaceMonocleEvent(currentWorkspace));
      else
        _bus.Emit(new ExitWorkspaceMonocleEvent(currentWorkspace));

      _containerService.ContainersToRedraw.Add(currentWorkspace);

      return CommandResponse.Fail;
    }
  }
}
