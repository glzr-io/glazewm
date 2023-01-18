using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;

using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class CenterCursorOnRectHandler : ICommandHandler<CenterCursorOnRectCommand>
  {
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnRectHandler(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnRectCommand command)
    {
      var enabled = _userConfigService.GeneralConfig.CursorFollowsFocus;
      if (!enabled)
        return CommandResponse.Ok;

      Rect TargetRect = command.TargetRect;

      // Calculate center point of focused window
      var centerX = TargetRect.X + (TargetRect.Width / 2);
      var centerY = TargetRect.Y + (TargetRect.Height / 2);
      SetCursorPos(centerX, centerY);
      return CommandResponse.Ok;
    }
  }
}
