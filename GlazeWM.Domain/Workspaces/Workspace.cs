using System;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Workspaces
{
  public class Workspace : SplitContainer
  {
    public string Name { get; set; }

    public string CustomDisplayName =>
      _userConfigService.UserConfig.Workspaces.First(w => w.Name == Name).CustomDisplayName ?? Name;

    private UserConfigService _userConfigService =
        ServiceLocator.Provider.GetRequiredService<UserConfigService>();

    private WorkspaceService _workspaceService =
        ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    private int OuterGap => _userConfigService.UserConfig.Gaps.OuterGap;

    /// <summary>
    /// Get height of bar after it's been automatically adjusted by DPI scaling.
    /// </summary>
    private int LogicalBarHeight
    {
      get
      {
        var barHeight = _userConfigService.UserConfig.Bar.Height;
        return Convert.ToInt32(barHeight * (Parent as Monitor).ScaleFactor);
      }
    }

    public override int Height => Parent.Height - (OuterGap * 2) - LogicalBarHeight;
    public override int Width => Parent.Width - (OuterGap * 2);
    public override int X => Parent.X + OuterGap;
    public override int Y => Parent.Y + OuterGap + (_userConfigService.UserConfig.Bar.Position == BarPosition.Top ? LogicalBarHeight : 0);

    /// <summary>
    /// Whether the workspace itself or a descendant container has focus.
    /// </summary>
    public bool HasFocus => _workspaceService.GetFocusedWorkspace() == this;

    /// <summary>
    /// Whether the workspace is currently displayed by the parent monitor.
    /// </summary>
    public bool IsDisplayed => (Parent as Monitor)?.DisplayedWorkspace == this;

    public Workspace(string name)
    {
      Name = name;
    }
  }
}
