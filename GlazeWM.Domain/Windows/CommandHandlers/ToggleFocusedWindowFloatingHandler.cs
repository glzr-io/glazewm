using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ToggleFocusedWindowFloatingHandler : ICommandHandler<ToggleFocusedWindowFloatingCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ToggleFocusedWindowFloatingHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ToggleFocusedWindowFloatingCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow is null || !_containerService.IsFocusSynced)
        return CommandResponse.Ok;

      _bus.Invoke(new ToggleFloatingCommand(focusedWindow));

      return CommandResponse.Ok;
    }
  }
}
