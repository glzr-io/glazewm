using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class MoveWorkspaceToMonitorCommand : Command
  {
    public Direction Direction { get; }

    public MoveWorkspaceToMonitorCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
