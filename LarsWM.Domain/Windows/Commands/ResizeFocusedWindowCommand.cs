using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowCommand : Command
  {
    public ResizeDirection ResizeDirection { get; }

    public ResizeFocusedWindowCommand(ResizeDirection resizeDirection)
    {
      ResizeDirection = resizeDirection;
    }
  }
}
