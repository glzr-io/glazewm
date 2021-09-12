using System.Collections.Generic;
using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.UserConfigs;

namespace LarsWM.Domain.Containers
{
  public class ContainerService
  {
    /// <summary>
    /// The root node of the container tree. Monitors are the children of the root node, followed
    /// by workspaces, then split containers/windows.
    /// </summary>
    public Container ContainerTree = new Container();

    /// <summary>
    /// Pending SplitContainers to redraw.
    /// </summary>
    public List<SplitContainer> SplitContainersToRedraw = new List<SplitContainer>();

    /// <summary>
    /// The currently focused container. This can either be a `Window` or a
    /// `Workspace` without any descendant windows.
    /// </summary>
    public Container FocusedContainer => ContainerTree.LastFocusedDescendant;

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

      if (parent.Layout == Layout.VERTICAL)
        return parent.Width;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return (int)(container.SizePercentage * (parent.Width - (innerGap * (parent.Children.Count - 1))));
    }

    /// <summary>
    /// Calculates the height of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateHeightOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.HORIZONTAL)
        return parent.Height;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return (int)(container.SizePercentage * (parent.Height - (innerGap * (parent.Children.Count - 1))));
    }

    /// <summary>
    /// Calculates the X coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateXOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.VERTICAL || container.Index == 0)
        return parent.X;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return container.PreviousSibling.X + container.PreviousSibling.Width + innerGap;
    }

    /// <summary>
    /// Calculates the Y coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateYOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.Layout == Layout.HORIZONTAL || container.Index == 0)
        return parent.Y;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return container.PreviousSibling.Y + container.PreviousSibling.Height + innerGap;
    }

    /// <summary>
    /// Traverse down a split container in search of a descendant in the given direction. For example,
    /// get the rightmost container for `Direction.RIGHT`.
    /// </summary>
    public Container GetDescendantInDirection(Container originContainer, Direction direction)
    {
      if (!(originContainer is SplitContainer))
        return originContainer;

      var layout = (originContainer as SplitContainer).Layout;

      // TODO: Need to correct focus stack after moving out a container from a vertical split container. With
      // the current implementation, the split container still references the moved out container.

      if (layout != direction.GetCorrespondingLayout())
        return GetDescendantInDirection(originContainer.LastFocusedChild, direction);

      if (direction == Direction.UP || direction == Direction.LEFT)
        return GetDescendantInDirection(originContainer.Children.Last(), direction);
      else
        return GetDescendantInDirection(originContainer.Children.First(), direction);
    }
  }
}
