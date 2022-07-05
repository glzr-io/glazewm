using System;
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

    private readonly UserConfigService _userConfigService =
        ServiceLocator.Provider.GetRequiredService<UserConfigService>();

    private readonly WorkspaceService _workspaceService =
        ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    public string DisplayName =>
      _userConfigService.GetWorkspaceConfigByName(Name).DisplayName ?? Name;

    public bool KeepAlive =>
      _userConfigService.GetWorkspaceConfigByName(Name).KeepAlive;

    /// <summary>
    /// Get height of bar after it's been automatically adjusted by DPI scaling.
    /// </summary>
    private int _logicalBarHeight
    {
      get
      {
        var barHeight = _userConfigService.UserConfig.Bar.Height;
        return Convert.ToInt32(barHeight * (Parent as Monitor).ScaleFactor);
      }
    }

    private int _yOffset
    {
      get
      {
        var barPosition = _userConfigService.UserConfig.Bar.Position;
        return barPosition == BarPosition.Top ? _logicalBarHeight : 0;
      }
    }

    private int _outerGap => _userConfigService.UserConfig.Gaps.OuterGap;

    public override int Height => Parent.Height - (_outerGap * 2) - _logicalBarHeight;
    public override int Width => Parent.Width - (_outerGap * 2);
    public override int X => Parent.X + _outerGap;
    public override int Y => Parent.Y + _outerGap + _yOffset;

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
