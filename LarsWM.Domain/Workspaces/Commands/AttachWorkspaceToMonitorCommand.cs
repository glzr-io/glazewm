using LarsWM.Domain.Monitors;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
{
  class AttachWorkspaceToMonitorCommand : Command
  {
    public Workspace Workspace { get; }
    public Monitor Monitor { get; }

    public AttachWorkspaceToMonitorCommand(Workspace workspace, Monitor monitor)
    {
      Workspace = workspace;
      Monitor = monitor;
    }
  }
}
