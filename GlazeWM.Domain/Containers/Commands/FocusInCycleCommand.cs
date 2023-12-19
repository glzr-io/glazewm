using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class FocusInCycleCommand : Command
  {
    public Direction Direction { get; }

    public FocusInCycleCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
