using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class MoveWorkspaceInDirectionCommand : Command
  {
    public Direction Direction { get; }

    public MoveWorkspaceInDirectionCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
