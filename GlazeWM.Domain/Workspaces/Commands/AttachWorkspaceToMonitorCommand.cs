using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
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
