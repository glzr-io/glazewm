using System.Collections.Generic;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  internal class UpdateWorkspacesFromConfigCommand : Command
  {
    public List<WorkspaceConfig> WorkspaceConfigs { get; }

    public UpdateWorkspacesFromConfigCommand(List<WorkspaceConfig> workspaceConfigs)
    {
      WorkspaceConfigs = workspaceConfigs;
    }
  }
}
