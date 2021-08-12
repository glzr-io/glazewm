using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
{
  class DisplayWorkspaceCommand : Command
  {
    public Workspace Workspace { get; }

    public DisplayWorkspaceCommand(Workspace workspace)
    {
      Workspace = workspace;
    }
  }
}
