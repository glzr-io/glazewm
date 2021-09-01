using System;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain.Workspaces
{
  public class Workspace : SplitContainer
  {
    public Guid Id = Guid.NewGuid();
    public string Name { get; set; }

    private UserConfigService _userConfigService =
        ServiceLocator.Provider.GetRequiredService<UserConfigService>();

    private ContainerService _containerService =
        ServiceLocator.Provider.GetRequiredService<ContainerService>();

    private WorkspaceService _workspaceService =
        ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    // TODO: Use bar height instead of hardcoded value.
    public override int Height => Parent.Height - (_userConfigService.UserConfig.Gaps.OuterGap * 2) - 50;
    public override int Width => Parent.Width - (_userConfigService.UserConfig.Gaps.OuterGap * 2);
    public override int X => Parent.X + _userConfigService.UserConfig.Gaps.OuterGap;
    // TODO: Use bar height instead of hardcoded value.
    public override int Y => Parent.Y + _userConfigService.UserConfig.Gaps.OuterGap + 50;

    /// <summary>
    /// Whether the workspace itself or a descendant container has focus.
    /// </summary>
    public bool HasFocus
    {
      get
      {
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
