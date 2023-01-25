using System.Diagnostics;
using System.Net.NetworkInformation;
using System.Text;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class CenterCursorOnRectHandler : ICommandHandler<CenterCursorOnRectCommand>
  {
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnRectHandler(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnRectCommand command)
    {
      var isEnabled = _userConfigService.GeneralConfig.CursorFollowsFocus;

      if (!isEnabled)
        return CommandResponse.Ok;

      var targetRect = command.TargetRect;

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);

      // Get the index of the interface with the best route to the remote address.
      var dwDestAddr = System.BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      uint dwBestIfIndex = 0;
      var result = GetBestInterface(dwDestAddr, ref dwBestIfIndex);
      if (result != 0)
        throw new NetworkInformationException(result);

      // Find a matching .NET interface object with the given index.
      foreach (var networkInterface in NetworkInterface.GetAllNetworkInterfaces())
        if (networkInterface.GetIPProperties().GetIPv4Properties().Index == dwBestIfIndex)
          Debug.WriteLine(networkInterface);

      return CommandResponse.Ok;
    }
  }
}
