using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class RemoveTitleBarHandler : ICommandHandler<RemoveTitleBarCommand>
  {
    private readonly Bus _bus;

    public RemoveTitleBarHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(RemoveTitleBarCommand command)
    {
      var window = command.Window;

      var result1 = WindowsApiService.SetWindowLong(
        window.Handle,
        WindowsApiService.GWLSTYLE,
        unchecked((int)WindowsApiService.WindowStyles.PopupWindow)
      );

      if (result1 == 0)
      {
        return CommandResponse.Fail;
      }

      uint preference = 2;
      var result2 = WindowsApiService.DwmSetWindowAttribute(
        window.Handle,
        (uint)WindowsApiService.DwmWindowAttribute.WindowCornerPreference,
        ref preference,
        sizeof(int)
      );

      if (result2 == 0)
      {
        return CommandResponse.Fail;
      }

      return CommandResponse.Ok;
    }
  }
}
