using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class DisplayWorkspaceCommand : Command
  {
    public Workspace Workspace { get; }

    public DisplayWorkspaceCommand(Workspace workspace)
    {
      Workspace = workspace;
    }
  }
}
