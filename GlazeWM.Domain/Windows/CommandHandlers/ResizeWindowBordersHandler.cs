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
      // TODO: Move default border delta into some sort of shared state.
      var defaultBorderDelta = new RectDelta(7, 0, 7, 7);
      windowToResize.BorderDelta = new RectDelta(
        defaultBorderDelta.Left + borderDelta.Left,
        defaultBorderDelta.Top + borderDelta.Top,
        defaultBorderDelta.Right + borderDelta.Right,
        defaultBorderDelta.Bottom + borderDelta.Bottom
      );

      // No need to redraw if window isn't tiling.
      if (windowToResize is not TilingWindow)
        return CommandResponse.Ok;

      _containerService.ContainersToRedraw.Add(windowToResize);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
