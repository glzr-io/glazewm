using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowCommand : Command
  {
    public ResizeDirection ResizeDirection { get; }
    public string ResizeAmount { get; }

    public ResizeFocusedWindowCommand(ResizeDirection resizeDirection, string resizeAmount)
    {
      ResizeDirection = resizeDirection;
      ResizeAmount = resizeAmount;
    }
  }
}
