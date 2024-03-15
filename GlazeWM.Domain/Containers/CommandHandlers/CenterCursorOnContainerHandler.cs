using System;
using System.Runtime.InteropServices;
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

      var targetRect = command.TargetContainer.ToRect();

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);

      var container = command.TargetContainer;

      if (container is Workspace)
      {
        var inputs = new INPUT[1];
        inputs[0].type = 0;
        inputs[0].data.mi.dx = centerX;
        inputs[0].data.mi.dy = centerY;
        inputs[0].data.mi.dwFlags = MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP;

        var result = SendInput(1, inputs, Marshal.SizeOf(typeof(INPUT)));
        if (result == 0)
        {
          throw new Exception("Error occurred while simulating mouse click. This is a bug.");
        }
      }

      return CommandResponse.Ok;
    }
  }
}
