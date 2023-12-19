using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class ToggleMaximizedHandler : ICommandHandler<ToggleMaximizedCommand>
  {
    public CommandResponse Handle(ToggleMaximizedCommand command)
    {
      var window = command.Window;

      if (window.HasWindowStyle(WindowStyles.Maximize))
        ShowWindowAsync(window.Handle, ShowWindowFlags.Restore);
      else
        ShowWindowAsync(window.Handle, ShowWindowFlags.Maximize);

      return CommandResponse.Ok;
    }
  }
}
