using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class CenterCursorOnContainerHandler : ICommandHandler<CenterCursorOnContainerCommand>
  {
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnContainerHandler(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnContainerCommand command)
    {
      var isEnabled = _userConfigService.GeneralConfig.CursorFollowsFocus;

      if (!isEnabled || command.TargetContainer.IsDetached() || command.TargetContainer.FocusIndex < 0)
        return CommandResponse.Ok;
      // in monocle mode, center cursor on the current workspace instead of targetContainer
      var targetWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(command.TargetContainer);
      if (targetWorkspace.isMonocle)
      {
        var rect = targetWorkspace.ToRect();
        var x = rect.X + (rect.Width / 2);
        var y = rect.Y + (rect.Height / 2);
        SetCursorPos(x, y);
        return CommandResponse.Ok;
      }

      var targetRect = command.TargetContainer.ToRect();

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);

      return CommandResponse.Ok;
    }
  }
}
