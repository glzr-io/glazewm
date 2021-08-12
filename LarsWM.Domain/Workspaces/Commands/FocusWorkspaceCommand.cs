using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
{
  public class FocusWorkspaceCommand : Command
  {
    public string WorkspaceName { get; }

    public FocusWorkspaceCommand(string workspaceName)
    {
      WorkspaceName = workspaceName;
    }
  }
}
