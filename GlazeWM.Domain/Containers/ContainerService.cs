using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers
{
  public class ContainerService
  {
    /// <summary>
    /// The root node of the container tree. Monitors are the children of the root node, followed
    /// by workspaces, then split containers/windows.
    /// </summary>
    public RootContainer ContainerTree = new();

    /// <summary>
    /// Containers (and their descendants) to redraw on the next invocation of
    /// `RedrawContainersCommand`.
    /// </summary>
    public List<Container> ContainersToRedraw = new();

    /// <summary>
    /// The currently focused container. This can either be a `Window` or a
    /// `Workspace` without any descendant windows.
    /// </summary>
    public Container FocusedContainer => ContainerTree.LastFocusedDescendant;

    /// <summary>
    /// Whether native focus will need to be reassigned to the WM's focused container.
    /// </summary>
    public bool HasPendingFocusSync { get; set; }

    /// <summary>
    /// Whether a tiling or floating container is currently focused.
    /// </summary>
    public FocusMode FocusMode => FocusedContainer is FloatingWindow ?
      FocusMode.Floating : FocusMode.Tiling;

    /// <summary>
    /// Name of the currently active binding mode (if one is active).
    /// </summary>
    public string ActiveBindingMode { get; set; }

    /// <summary>
    /// If set, this container overrides the target container to set focus to on the next
    /// focus window event (ie. `EVENT_SYSTEM_FOREGROUND`).
    /// </summary>
    public Container PendingFocusContainer { get; set; }

    private readonly UserConfigService _userConfigService;

    public ContainerService(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public Container GetContainerById(Guid id)
    {
      return ContainerTree.SelfAndDescendants.FirstOrDefault(container => container.Id == id);
    }

    /// <summary>
    /// Calculates the width of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int GetWidthOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.TilingDirection == TilingDirection.Vertical)
        return parent.Width;

      var innerGap = UnitsHelper.TrimUnits(_userConfigService.GapsConfig.InnerGap);
      var resizableSiblings = container.SelfAndSiblingsOfType<IResizable>();

      return (int)((container as IResizable).SizePercentage
        * (parent.Width - (innerGap * (resizableSiblings.Count() - 1))));
    }

    /// <summary>
    /// Calculates the height of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int GetHeightOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      if (parent.TilingDirection == TilingDirection.Horizontal)
        return parent.Height;

      var innerGap = UnitsHelper.TrimUnits(_userConfigService.GapsConfig.InnerGap);
      var resizableSiblings = container.SelfAndSiblingsOfType<IResizable>();

      return (int)((container as IResizable).SizePercentage
        * (parent.Height - (innerGap * (resizableSiblings.Count() - 1))));
    }

    /// <summary>
    /// Calculates the X coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int GetXOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      var isFirstOfType = container.SelfAndSiblingsOfType<IResizable>().First() == container;

      if (parent.TilingDirection == TilingDirection.Vertical || isFirstOfType)
        return parent.X;

      var innerGap = UnitsHelper.TrimUnits(_userConfigService.GapsConfig.InnerGap);

      return container.PreviousSiblingOfType<IResizable>().X
        + container.PreviousSiblingOfType<IResizable>().Width
        + innerGap;
    }

    /// <summary>
    /// Calculates the Y coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int GetYOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      var isFirstOfType = container.SelfAndSiblingsOfType<IResizable>().First() == container;

      if (parent.TilingDirection == TilingDirection.Horizontal || isFirstOfType)
        return parent.Y;

      var innerGap = UnitsHelper.TrimUnits(_userConfigService.GapsConfig.InnerGap);

      return container.PreviousSiblingOfType<IResizable>().Y
        + container.PreviousSiblingOfType<IResizable>().Height
        + innerGap;
    }

    /// <summary>
    /// Traverse down a container in search of a descendant in the given direction. For example, get
    /// the rightmost container for `Direction.RIGHT`. Returns the `originContainer` if no suitable
    /// descendants are found. Any non-tiling containers are ignored.
    /// </summary>
    public Container GetDescendantInDirection(Container originContainer, Direction direction)
    {
      var isDescendable =
        originContainer is SplitContainer &&
        originContainer.ChildrenOfType<IResizable>().Any();

      if (!isDescendable)
        return originContainer;

      var tilingDirection = (originContainer as SplitContainer).TilingDirection;

      if (tilingDirection != direction.GetTilingDirection())
        return GetDescendantInDirection(
          originContainer.LastFocusedChildOfType<IResizable>(),
          direction
        );
      else if (direction is Direction.Up or Direction.Left)
        return GetDescendantInDirection(
          originContainer.ChildrenOfType<IResizable>().First(),
          direction
        );
      else
        return GetDescendantInDirection(
          originContainer.ChildrenOfType<IResizable>().Last(),
          direction
        );
    }

    /// <summary>
    /// Get the lowest container in the tree that has both `containerA` and `containerB` as
    /// descendants.
    /// </summary>
    public static Container GetLowestCommonAncestor(Container containerA, Container containerB)
    {
      var ancestorA = containerA;

      // Traverse upwards from container A.
      while (ancestorA != null)
      {
        var ancestorB = containerB;

        // Traverse upwards from container B.
        while (ancestorB != null)
        {
          if (ancestorA == ancestorB)
            return ancestorA;

          ancestorB = ancestorB.Parent;
        }

        ancestorA = ancestorA.Parent;
      }

      throw new Exception("No common ancestor between containers. This is a bug.");
    }
  }
}
