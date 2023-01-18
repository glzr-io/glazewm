using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal sealed class MoveWindowToWorkspaceCommand : Command
  {
    public Window WindowToMove { get; }
    public string WorkspaceName { get; }

    public MoveWindowToWorkspaceCommand(Window windowToMove, string workspaceName)
    {
      WindowToMove = windowToMove;
      WorkspaceName = workspaceName;
    }
  }
}
