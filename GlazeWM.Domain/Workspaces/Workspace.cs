using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Workspaces
{
  public class Workspace : SplitContainer
  {
    public string Name { get; set; }

    private UserConfigService _userConfigService =
        ServiceLocator.Provider.GetRequiredService<UserConfigService>();

    private ContainerService _containerService =
        ServiceLocator.Provider.GetRequiredService<ContainerService>();

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

    public IEnumerable<Container> TilingChildren => Children.Where(
      container => container is TilingWindow || container is SplitContainer
    );

    public IEnumerable<Container> FloatingChildren => Children.Where(
      container => container is FloatingWindow
    );

    public IEnumerable<Container> MinimizedChildren => Children.Where(
      container => container is MinimizedWindow
    );

    public IEnumerable<Container> MaximizedChildren => Children.Where(
      container => container is MaximizedWindow
    );

    public IEnumerable<Container> FullscreenChildren => Children.Where(
      container => container is FullscreenWindow
    );

    public override int Height => Parent.Height - (OuterGap * 2) - LogicalBarHeight;
    public override int Width => Parent.Width - (OuterGap * 2);
    public override int X => Parent.X + OuterGap;
    public override int Y => Parent.Y + OuterGap + LogicalBarHeight;

    /// <summary>
    /// Whether the workspace itself or a descendant container has focus.
    /// </summary>
    public bool HasFocus
    {
      get
      {
        // TODO: Refactor this to use `_workspaceService.GetFocusedWorkspace()`.
        var focusedContainer = _containerService.FocusedContainer;

        if (focusedContainer == null)
          return false;

        var focusedWorkspace = _workspaceService.GetWorkspaceFromChildContainer(focusedContainer);

        if (focusedWorkspace != this && focusedContainer != this)
          return false;

        return true;
      }
    }

    /// <summary>
    /// Whether the workspace is currently displayed by the parent monitor.
    /// </summary>
    public bool IsDisplayed
    {
      get
      {
        return (Parent as Monitor)?.DisplayedWorkspace == this;
      }
    }

    public Workspace(string name)
    {
      Name = name;
    }
  }
}
