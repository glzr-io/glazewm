using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class ActivateWorkspaceCommand : Command
  {
    public string WorkspaceName { get; }
    public Monitor TargetMonitor { get; }

    public ActivateWorkspaceCommand(string workspaceName, Monitor targetMonitor)
    {
      WorkspaceName = workspaceName;
      TargetMonitor = targetMonitor;
    }
  }
}
