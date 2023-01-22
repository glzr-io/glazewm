using GlazeWM.Domain.Monitors;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal sealed class ActivateWorkspaceCommand : Command
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
