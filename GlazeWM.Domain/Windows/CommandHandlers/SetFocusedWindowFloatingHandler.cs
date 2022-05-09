using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class SetFocusedWindowFloatingHandler : ICommandHandler<SetFocusedWindowFloatingCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public SetFocusedWindowFloatingHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(SetFocusedWindowFloatingCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow == null || !_containerService.IsFocusSynced)
        return CommandResponse.Ok;

      _bus.Invoke(new SetFloatingCommand(focusedWindow));

      return CommandResponse.Ok;
    }
  }
}
