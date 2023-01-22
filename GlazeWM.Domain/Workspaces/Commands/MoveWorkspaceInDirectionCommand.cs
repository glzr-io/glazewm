using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal sealed class MoveWorkspaceInDirectionCommand : Command
  {
    // TODO: Add argument for workspace to move instead of assuming the focused workspace.
    public Direction Direction { get; }

    public MoveWorkspaceInDirectionCommand(Direction direction)
    {
      Direction = direction;
    }
  }
}
