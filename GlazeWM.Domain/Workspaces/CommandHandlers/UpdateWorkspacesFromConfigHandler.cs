using System.Linq;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class UpdateWorkspacesFromConfigHandler :
    ICommandHandler<UpdateWorkspacesFromConfigCommand>
  {
    private readonly Bus _bus;
    private readonly WorkspaceService _workspaceService;
    private readonly UserConfigService _userConfigService;

    public UpdateWorkspacesFromConfigHandler(
      Bus bus,
      WorkspaceService workspaceService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(UpdateWorkspacesFromConfigCommand command)
    {
      var workspaceConfigs = command.WorkspaceConfigs;
      var workspaces = _workspaceService.GetActiveWorkspaces();

      foreach (var workspace in workspaces)
      {
        var workspaceConfig = workspaceConfigs.Find(config => config.Name == workspace.Name);

        if (workspaceConfig is null)
        {
          var monitor = workspace.Parent as Monitor;
          // TODO: Move this to a dedicated `WorkspaceService` method.
          var inactiveWorkspaceName = _workspaceService.GetInactiveWorkspaceNameForMonitor(monitor) ??
            _workspaceService.GetInactiveWorkspaceNamesNotDedicatedToAMonitor().ElementAtOrDefault(0) ??
            _workspaceService.GetInactiveWorkspaceNames().ElementAtOrDefault(0);

          if (inactiveWorkspaceName is null)
            throw new FatalUserException("At least 1 workspace is required per monitor.");

          var inactiveWorkspaceConfig =
            _userConfigService.GetWorkspaceConfigByName(inactiveWorkspaceName);

          workspace.Name = inactiveWorkspaceName;
        }

        // TODO: Update `DisplayName` and `KeepAlive` once they are changed to properties.
      }

      return CommandResponse.Ok;
    }
  }
}
