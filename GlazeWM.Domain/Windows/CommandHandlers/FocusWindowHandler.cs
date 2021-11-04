using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class FocusWindowHandler : ICommandHandler<FocusWindowCommand>
  {
    public FocusWindowHandler()
    {
    }

    public CommandResponse Handle(FocusWindowCommand command)
    {
      var window = command.Window;

      // Set as foreground window if it's not already set. This will trigger `EVENT_SYSTEM_FOREGROUND`
      // window event and its handler.
      KeybdEvent(0, 0, 0, 0);
      SetForegroundWindow(window.Hwnd);

      return CommandResponse.Ok;
    }
  }
}
