using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ResizeWindowBordersHandler : ICommandHandler<ResizeWindowBordersCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ResizeWindowBordersHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeWindowBordersCommand command)
    {
      var borderDelta = command.BorderDelta;
      var windowToResize = command.WindowToResize;

      // Set the new border delta of the window.
      windowToResize.BorderDelta = new RectDelta(
        borderDelta.DeltaLeft,
        borderDelta.DeltaTop,
        borderDelta.DeltaRight,
        borderDelta.DeltaBottom
      );

      if (windowToResize is not TilingWindow)
        return CommandResponse.Ok;

      // Only redraw the window if it's tiling.
      _containerService.ContainersToRedraw.Add(windowToResize);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
