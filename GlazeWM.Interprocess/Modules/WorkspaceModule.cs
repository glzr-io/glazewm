using System.Linq;
using System.Threading.Tasks;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Interprocess.Models;
using Qmmands;

namespace GlazeWM.Interprocess.Modules
{
  [Group("workspace")]
  internal sealed class WorkspaceModule : InterprocessModule
  {
    [Command("getall")]
    [Description("Gets all active workspaces.")]
    public Task GetAllAsync()
    {
      var workspaces = MonitorService
                       .GetMonitors()
                       .Select(monitor => monitor.Children.OfType<Workspace>()
                                                 .Select(workspace => new WorkspaceInfo(workspace))
                                                 .ToArray());

      Context.Server.SendToSession(new { workspaces }, SessionId);

      return Task.CompletedTask;
    }

    [Command("get")]
    [Description("Gets an active workspace by name.")]
    public Task GetAsync([Remainder] string name)
    {
      var workspace = new WorkspaceInfo(WorkspaceService.GetActiveWorkspaceByName(name));

      Context.Server.SendToSession(workspace, SessionId);

      return Task.CompletedTask;
    }
  }
}
