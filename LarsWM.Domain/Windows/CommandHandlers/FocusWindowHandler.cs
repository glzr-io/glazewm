using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
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

    public dynamic Handle(FocusWindowCommand command)
    {
      var window = command.Window;

      if (window == _containerService.FocusedContainer)
        return CommandResponse.Ok;

      // Create a focus stack pointing to the newly focused window.
      _bus.Invoke(new CreateFocusStackCommand(window));

      SetForegroundWindow(window.Hwnd);

      return new CommandResponse(true, window.Id);
    }
  }
}
