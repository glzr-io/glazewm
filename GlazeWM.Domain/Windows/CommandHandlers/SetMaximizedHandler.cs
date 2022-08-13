using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class SetMaximizedHandler : ICommandHandler<SetMaximizedCommand>
  {
    public CommandResponse Handle(SetMaximizedCommand command)
    {
      var window = command.Window;

      ShowWindow(window.Hwnd, ShowWindowCommands.MAXIMIZE);

      return CommandResponse.Ok;
    }
  }
}
