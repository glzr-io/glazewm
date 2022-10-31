using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  internal class FocusWorkspaceSequenceHandler : ICommandHandler<FocusWorkspaceSequenceCommand>
  {
    private readonly Bus _bus;
    private readonly UserConfigService _userConfigService;
    private readonly WorkspaceService _workspaceService;

    public FocusWorkspaceSequenceHandler(
      Bus bus,
      UserConfigService userConfigService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(FocusWorkspaceSequenceCommand command)
    {
      // Get the currently focused workspace.
      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();

      var workspacesConfigs = _userConfigService.WorkspaceConfigs;

      var focusWorkspaceConfig = workspacesConfigs
        .Where(config => config.Name == focusedWorkspace.Name)
        .First();

      // Get index config current focused workspace.
      var currentIndex = workspacesConfigs.IndexOf(focusWorkspaceConfig);

      var configLength = workspacesConfigs.Count();

      // Calculate index requested workspace.
      var newIndex = command.Direction switch
      {
        Sequence.PREVIOUS => currentIndex == 0 ? configLength - 1 : currentIndex - 1,
        Sequence.NEXT => currentIndex == configLength - 1 ? 0 : currentIndex + 1,
        _ => currentIndex,
      };

      // If workspace changed, focus on new workspace.
      if (newIndex != currentIndex)
      {
        var newWorkspaceName = workspacesConfigs[newIndex].Name;
        _bus.Invoke(new FocusWorkspaceCommand(newWorkspaceName));
      }

      return CommandResponse.Ok;
    }
  }
}
