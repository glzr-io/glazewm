using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class FocusInDirectionCommand : Command
  {
    public Direction Direction { get; }

    public FocusInDirectionCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
