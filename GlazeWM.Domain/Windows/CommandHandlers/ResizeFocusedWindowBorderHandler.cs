using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ResizeFocusedWindowBorderHandler : ICommandHandler<ResizeFocusedWindowBordersCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ResizeFocusedWindowBorderHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeFocusedWindowBordersCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      // Only redraw the window if it's tiling.
      if (focusedWindow is TilingWindow)
      {
        _containerService.ContainersToRedraw.Add(focusedWindow);
        _bus.Invoke(new RedrawContainersCommand());
      }

      return CommandResponse.Ok;
    }
  }
}
