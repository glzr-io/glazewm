using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
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
