using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;

using GlazeWM.Infrastructure.Bussing;

using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class CenterCursorOnContainerHandler : ICommandHandler<CenterCursorOnContainerCommand>
  {
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnContainerHandler(ContainerService containerService, UserConfigService userConfigService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnContainerCommand command)
    {
      var enabled = _userConfigService.GeneralConfig.CursorFollowsFocus;
      if (!enabled)
        return CommandResponse.Ok;

      var target = command.TargetContainer;

      // Calculate center point of focused window
      var centerX = target.X + target.Width / 2;
      var centerY = target.Y + target.Height / 2;

      SetCursorPos(centerX, centerY);

      return CommandResponse.Ok;
    }
  }
}
