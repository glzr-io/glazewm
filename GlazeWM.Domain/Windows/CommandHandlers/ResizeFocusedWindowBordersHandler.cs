using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ResizeFocusedWindowBordersHandler : ICommandHandler<ResizeFocusedWindowBordersCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ResizeFocusedWindowBordersHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeFocusedWindowBordersCommand command)
    {
      var borderDelta = command.ResizeDimensions;
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      focusedWindow.InvisibleBorders = new WindowRect()
      {
        Left = focusedWindow.InvisibleBorders.Left + borderDelta.DeltaLeft,
        Right = focusedWindow.InvisibleBorders.Right + borderDelta.DeltaRight,
        Top = focusedWindow.InvisibleBorders.Top + borderDelta.DeltaTop,
        Bottom = focusedWindow.InvisibleBorders.Bottom + borderDelta.DeltaBottom,
      };

      if (!(focusedWindow is TilingWindow))
        return CommandResponse.Ok;

      // Only redraw the window if it's tiling.
      _containerService.ContainersToRedraw.Add(focusedWindow);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
