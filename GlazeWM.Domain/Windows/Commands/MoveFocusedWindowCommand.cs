using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
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
