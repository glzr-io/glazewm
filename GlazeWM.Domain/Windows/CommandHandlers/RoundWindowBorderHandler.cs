using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class RoundWindowBorderHandler : ICommandHandler<RoundWindowBorderCommand>
  {
    public CommandResponse Handle(RoundWindowBorderCommand command)
    {
      uint cornerPreference = 0x2;
      var target = command.TargetWindow;
      _ = WindowsApiService.DwmSetWindowAttribute(target.Handle, 0x21, ref cornerPreference, sizeof(uint));

      return CommandResponse.Ok;
    }
  }
}
