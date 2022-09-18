using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ToggleMaximizedHandler : ICommandHandler<ToggleMaximizedCommand>
  {
    public CommandResponse Handle(ToggleMaximizedCommand command)
    {
      var window = command.Window;

      if (window.HasWindowStyle(WS.WS_MAXIMIZE))
        ShowWindow(window.Handle, ShowWindowCommands.RESTORE);
      else
        ShowWindow(window.Handle, ShowWindowCommands.MAXIMIZE);

      return CommandResponse.Ok;
    }
  }
}
