using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
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
