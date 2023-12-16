using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Containers;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class ToggleMonocleHandler : ICommandHandler<ToggleMonocleCommand>
  {
    private readonly ContainerService _containerService;

    public ToggleMonocleHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(ToggleMonocleCommand command)
    {
      var window = command.Window;

      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);
      workspace.isMonocle = !workspace.isMonocle;

      _containerService.ContainersToRedraw.Add(workspace);

      return CommandResponse.Ok;
    }
  }
}
