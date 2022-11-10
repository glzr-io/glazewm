using System.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class FocusWorkspaceRecentHandler : ICommandHandler<FocusWorkspaceRecentCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;
    private readonly WorkspaceService _workspaceService;

    public FocusWorkspaceRecentHandler(
      Bus bus,
      UserConfigService userConfigService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(FocusWorkspaceRecentCommand command)
    {
      var recentWorkspace = _workspaceService.PopRecentWorkspace();
      var workspaceConfigs = _userConfigService.WorkspaceConfigs;

      while (recentWorkspace != null)
      {
        // Validate that workspace are still available
        if (workspaceConfigs.Any(workspace => workspace.Name == recentWorkspace.Name))
        {
          // Focus workspace
          _bus.Invoke(new FocusWorkspaceCommand(recentWorkspace.Name));
          // Remove last entry so as not to get stuck 
          _workspaceService.PopRecentWorkspace();

          return CommandResponse.Ok;
        }

        recentWorkspace = _workspaceService.PopRecentWorkspace();
      }

      return CommandResponse.Fail;
    }
  }
}
