using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  public class FocusMonitorCommand : Command
  {
    public Direction Direction { get; }

    public FocusMonitorCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
