using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class SetMinimizedHandler : ICommandHandler<SetMinimizedCommand>
  {
    public CommandResponse Handle(SetMinimizedCommand command)
    {
      var window = command.Window;

      ShowWindow(window.Hwnd, ShowWindowCommands.MINIMIZE);

      return CommandResponse.Ok;
    }
  }
}
