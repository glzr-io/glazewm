using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class MoveWindowCommand : Command
  {
    public Window WindowToMove { get; }
    public Direction Direction { get; }

    public MoveWindowCommand(Window windowToMove, Direction direction)
    {
      WindowToMove = windowToMove;
      Direction = direction;
    }
  }
}
