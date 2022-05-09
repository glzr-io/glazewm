using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class DeactivateWorkspaceCommand : Command
  {
    public Workspace Workspace { get; }

    public DeactivateWorkspaceCommand(Workspace workspace)
    {
      Workspace = workspace;
    }
  }
}
