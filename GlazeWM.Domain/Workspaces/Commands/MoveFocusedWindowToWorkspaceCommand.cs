using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  class MoveFocusedWindowToWorkspaceCommand : Command
  {
    public string WorkspaceName { get; }

    public MoveFocusedWindowToWorkspaceCommand(string workspaceName)
    {
      WorkspaceName = workspaceName;
    }
  }
}
