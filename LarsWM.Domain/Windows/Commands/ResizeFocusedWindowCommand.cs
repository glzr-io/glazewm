using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowCommand : Command
  {
    public Direction Direction { get; }

    public ResizeFocusedWindowCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
