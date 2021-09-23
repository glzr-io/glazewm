using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
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

      // Update focused container state if it's not already updated.
      if (_containerService.FocusedContainer != window)
      {
        _bus.Invoke(new SetFocusedDescendantCommand(window));
        _bus.RaiseEvent(new FocusChangedEvent(window));
      }

      // Set as foreground window if it's not already set. This will trigger `EVENT_SYSTEM_FOREGROUND`
      // window event and its handler.
      if (window.Hwnd != GetForegroundWindow())
      {
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow(window.Hwnd);
      }

      return CommandResponse.Ok;
    }
  }
}
