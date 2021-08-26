using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
{
  public class MoveFocusedWindowCommand : Command
  {
    public Direction Direction { get; }

    public MoveFocusedWindowCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
