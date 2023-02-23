using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class SetWindowSizeCommand : Command
  {
    public Window WindowToResize { get; }
    public ResizeDimension DimensionToResize { get; }
    public string ResizeAmount { get; }

    public SetWindowSizeCommand(
      Window windowToResize,
      ResizeDimension dimensionToResize,
      string resizeAmount)
    {
      WindowToResize = windowToResize;
      DimensionToResize = dimensionToResize;
      ResizeAmount = resizeAmount;
    }
  }
}
