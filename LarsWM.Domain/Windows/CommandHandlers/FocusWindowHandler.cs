using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class FocusWindowHandler : ICommandHandler<FocusWindowCommand>
  {
    private ContainerService _containerService;
    private Bus _bus;

    public FocusWindowHandler(ContainerService containerService, Bus bus)
    {
      _containerService = containerService;
      _bus = bus;
    }

    public CommandResponse Handle(FocusWindowCommand command)
    {
      var window = command.Window;

      // Set as foreground window, triggering `EVENT_SYSTEM_FOREGROUND` and its event handler.
      SetForegroundWindow(window.Hwnd);

      return CommandResponse.Ok;
    }
  }
}
