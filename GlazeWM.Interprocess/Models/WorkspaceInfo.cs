using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;

namespace GlazeWM.Interprocess.Models
{
  public sealed class WorkspaceInfo
  {
    public string Id { get; }

    public string DisplayName { get; }

    public MonitorInfo Monitor { get; }

    public bool Focused { get; }

    public List<WindowInfo> Windows { get; }

    public WorkspaceInfo(Workspace workspace)
    {
      Id = workspace.Id;
      DisplayName = workspace.DisplayName;
      Monitor = new((Monitor)workspace.Parent);
      Focused = workspace.HasFocus;
      Windows = workspace.Children
                         .OfType<Window>()
                         .Select(window => new WindowInfo(window))
                         .ToList();
    }
  }
}
