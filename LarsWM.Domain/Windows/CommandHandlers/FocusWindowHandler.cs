using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Containers.Events;
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

      // Update focused container state.
      _bus.Invoke(new SetFocusedDescendantCommand(window));
      _bus.RaiseEvent(new FocusChangedEvent(window));

      // Set as foreground window if it's not already set.
      if (window.Hwnd != GetForegroundWindow())
        SetForegroundWindow(window.Hwnd);

      return CommandResponse.Ok;
    }
  }
}
