using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowCommand : Command
  {
    public ResizeDimension DimensionToResize { get; }
    public string ResizeAmount { get; }

    public ResizeFocusedWindowCommand(ResizeDimension dimensionToResize, string resizeAmount)
    {
      DimensionToResize = dimensionToResize;
      ResizeAmount = resizeAmount;
    }
  }
}
