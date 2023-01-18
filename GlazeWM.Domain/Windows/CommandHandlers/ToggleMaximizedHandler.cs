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
        ShowWindow(window.Handle, ShowWindowFlags.Restore);
      else
        ShowWindow(window.Handle, ShowWindowFlags.Maximize);

      return CommandResponse.Ok;
    }
  }
}
