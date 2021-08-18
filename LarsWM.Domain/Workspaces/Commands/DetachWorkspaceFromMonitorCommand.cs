using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
{
  class DetachWorkspaceFromMonitorCommand : Command
  {
    public Workspace Workspace { get; }

    public DetachWorkspaceFromMonitorCommand(Workspace workspace)
    {
      Workspace = workspace;
    }
  }
}
