using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class SetWindowSizeHandler : ICommandHandler<SetWindowSizeCommand>
  {
    private readonly Bus _bus;

    public SetWindowSizeHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(SetWindowSizeCommand command)
    {
      // calculate the delta required to achivee the desired window size
      var resizeAmount = CalculateResizeAmount(command);

      // invoke the resize command with the calculated delta
      _bus.Invoke(new ResizeWindowCommand(command.WindowToResize, command.DimensionToResize, resizeAmount));

      return CommandResponse.Ok;
    }

    private static string CalculateResizeAmount(SetWindowSizeCommand command)
    {
      // get the parent dimension (width or height)
      var parentSize = command.DimensionToResize switch
      {
        ResizeDimension.Width => command.WindowToResize.Parent.Width,
        ResizeDimension.Height => command.WindowToResize.Parent.Height,
        _ => 0,
      };

      // get the current window dimension (width or height)
      var actualSize = command.DimensionToResize switch
      {
        ResizeDimension.Width => command.WindowToResize.Width,
        ResizeDimension.Height => command.WindowToResize.Height,
        _ => 0,
      };

      // calculate the desired size (based on parent size)
      var desiredSize = parentSize * ResizeParsingService.ParseResizeAmount(
          command.WindowToResize,
          command.DimensionToResize,
          command.ResizeAmount);

      // calculate the delta required to achieve the desired size
      var resizeDelta = desiredSize - actualSize;

      // return delta (in px)
      return resizeDelta > 0 ? $"+{resizeDelta}px" : $"{resizeDelta}px";
    }
  }
}
