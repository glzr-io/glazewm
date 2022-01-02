using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  class ActivateWorkspaceCommand : Command
  {
    public string WorkspaceName { get; private set; }
    public Monitor TargetMonitor { get; private set; }

    public ActivateWorkspaceCommand(string workspaceName, Monitor targetMonitor)
    {
      WorkspaceName = workspaceName;
      TargetMonitor = targetMonitor;
    }
  }
}
