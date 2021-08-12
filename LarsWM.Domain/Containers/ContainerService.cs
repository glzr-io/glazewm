using System.Collections.Generic;
using System.Diagnostics;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.UserConfigs;

namespace LarsWM.Domain.Containers
{
  public class ContainerService
  {
    /// <summary>
    /// List of trees consisting of containers. The root nodes are the monitors,
    /// followed by workspaces, then split containers/windows.
    /// </summary>
    // TODO: Rename to ContainerTrees/ContainerForest
    public List<Container> ContainerTree = new List<Container>();

    /// <summary>
    /// Pending SplitContainers to redraw.
    /// </summary>
    public List<SplitContainer> SplitContainersToRedraw = new List<SplitContainer>();

    private UserConfigService _userConfigService;

    public ContainerService(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    /// <summary>
    /// Calculates the width of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateWidthOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.Vertical)
        return parent.Width;

      var innerGap = _userConfigService.UserConfig.InnerGap;

      return (int)(container.SizePercentage * (parent.Width - (innerGap * (parent.Children.Count - 1))));
    }

    /// <summary>
    /// Calculates the height of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateHeightOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.Horizontal)
        return parent.Height;

      var innerGap = _userConfigService.UserConfig.InnerGap;

      return (int)(container.SizePercentage * (parent.Height - (innerGap * (parent.Children.Count - 1))));
    }

    /// <summary>
    /// Calculates the X coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateXOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.Vertical)
        return parent.X;

      var selfAndSiblings = parent.Children;
      var index = selfAndSiblings.IndexOf(container);

      if (index == 0)
        return parent.X;

      var innerGap = _userConfigService.UserConfig.InnerGap;
      var previousSibling = selfAndSiblings[index - 1];

      return previousSibling.X + previousSibling.Width + innerGap;
    }

    /// <summary>
    /// Calculates the Y coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateYOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.Horizontal)
        return parent.Y;

      var selfAndSiblings = parent.Children;
      var index = selfAndSiblings.IndexOf(container);

      if (index == 0)
        return parent.Y;

      var innerGap = _userConfigService.UserConfig.InnerGap;
      var previousSibling = selfAndSiblings[index - 1];

      return previousSibling.Y + previousSibling.Height + innerGap;
    }
  }
}
