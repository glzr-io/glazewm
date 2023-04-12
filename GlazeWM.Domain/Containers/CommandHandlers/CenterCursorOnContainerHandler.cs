using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
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

      if (!isEnabled || command.TargetContainer == null)
        return CommandResponse.Ok;

      if (command.TargetContainer.IsDetached())
        return CommandResponse.Ok;

      var targetRect = command.TargetContainer.ToRect();

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);

      return CommandResponse.Ok;
    }
  }
}
