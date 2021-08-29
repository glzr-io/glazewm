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

    public dynamic Handle(FocusWindowCommand command)
    {
      var window = command.Window;

      if (window == _containerService.FocusedContainer)
        return CommandResponse.Ok;

      _containerService.FocusedContainer = window;

      // Adjust focus order of ancestors.
      _bus.Invoke(new SetFocusedDescendantCommand(window));

      SetForegroundWindow(window.Hwnd);

      _bus.RaiseEvent(new FocusChangedEvent(window));

      return new CommandResponse(true, window.Id);
    }
  }
}
