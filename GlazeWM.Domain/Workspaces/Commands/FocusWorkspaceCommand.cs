using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
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
