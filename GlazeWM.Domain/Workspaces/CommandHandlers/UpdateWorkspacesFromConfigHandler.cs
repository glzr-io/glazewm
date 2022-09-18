using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Exceptions;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class UpdateWorkspacesFromConfigHandler :
    ICommandHandler<UpdateWorkspacesFromConfigCommand>
  {
    private readonly WorkspaceService _workspaceService;

    public UpdateWorkspacesFromConfigHandler(WorkspaceService workspaceService)
    {
      _workspaceService = workspaceService;
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
          var inactiveWorkspaceConfig = _workspaceService.GetWorkspaceConfigToActivate(monitor);

          if (inactiveWorkspaceConfig is null)
            throw new FatalUserException("At least 1 workspace is required per monitor.");

          workspace.Name = inactiveWorkspaceConfig.Name;
        }

        // TODO: Update `DisplayName` and `KeepAlive` once they are changed to properties.
      }

      return CommandResponse.Ok;
    }
  }
}
