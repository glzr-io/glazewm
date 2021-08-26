using System.Collections.Generic;
using System.Linq;
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

    /// <summary>
    /// The currently focused container. This can either be a `Window` or a
    /// `Workspace` without any descendant windows.
    /// </summary>
    public Container FocusedContainer { get; set; } = null;

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

      if (parent.Layout == Layout.Vertical || container.Index == 0)
        return parent.X;

      var innerGap = _userConfigService.UserConfig.InnerGap;

      return container.PreviousSibling.X + container.PreviousSibling.Width + innerGap;
    }

    /// <summary>
    /// Calculates the Y coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateYOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.Horizontal || container.Index == 0)
        return parent.Y;

      var innerGap = _userConfigService.UserConfig.InnerGap;

      return container.PreviousSibling.Y + container.PreviousSibling.Height + innerGap;
    }

    /// <summary>
    /// Traverse down a split container in search of a `Window` in the inverse direction
    /// of the given `Direction`. For example, get the rightmost Window for `Direction.LEFT`.
    /// </summary>
    public Container GetDescendantInDirection(Container originContainer, Direction direction)
    {
      if (!(originContainer is SplitContainer))
        return originContainer;

      var layout = (originContainer as SplitContainer).Layout;

      var doesNotMatchDirection =
        (layout == Layout.Vertical && (direction == Direction.LEFT || direction == Direction.RIGHT)) ||
        (layout == Layout.Horizontal && (direction == Direction.UP || direction == Direction.DOWN));

      if (doesNotMatchDirection)
        return GetDescendantInDirection(originContainer.LastFocusedContainer, direction);

      if (direction == Direction.UP || direction == Direction.LEFT)
        return GetDescendantInDirection(originContainer.Children.First(), direction);
      else
        return GetDescendantInDirection(originContainer.Children.Last(), direction);
    }
  }
}
