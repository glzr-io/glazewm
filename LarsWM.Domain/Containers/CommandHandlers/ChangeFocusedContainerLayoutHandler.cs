using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class ChangeFocusedContainerLayoutHandler : ICommandHandler<ChangeFocusedContainerLayoutCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public ChangeFocusedContainerLayoutHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ChangeFocusedContainerLayoutCommand command)
    {
      var focusedContainer = _containerService.FocusedContainer;
      var foregroundWindow = GetForegroundWindow();

      // Ignore cases where focused container is a window that's not in foreground.
      if (focusedContainer is Window && foregroundWindow != (focusedContainer as Window).Hwnd)
        return CommandResponse.Ok;

      _bus.Invoke(new ChangeContainerLayoutCommand(focusedContainer, command.NewLayout));

      return CommandResponse.Ok;
    }
  }
}
