using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
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
