using System.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class FocusWorkspaceToggleHandler : ICommandHandler<FocusWorkspaceToggleCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;
    private readonly WorkspaceService _workspaceService;

    public FocusWorkspaceToggleHandler(
      Bus bus,
      UserConfigService userConfigService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(FocusWorkspaceToggleCommand command)
    {
      var mostRecentWorkspace = _workspaceService.MostRecentWorkspace;
      var currentWorkspace = _workspaceService.GetFocusedWorkspace();
      var workspaceConfigs = _userConfigService.WorkspaceConfigs;

      if (mostRecentWorkspace != null)
      {
        // Validate that workspace are still available
        if (workspaceConfigs.Any(workspace => workspace.Name == mostRecentWorkspace.Name))
        {
          // Focus workspace
          _bus.Invoke(new FocusWorkspaceCommand(mostRecentWorkspace.Name));
          // Update most recent workspace
          _workspaceService.MostRecentWorkspace = currentWorkspace;

          return CommandResponse.Ok;
        }
      }

      return CommandResponse.Fail;
    }
  }
}
