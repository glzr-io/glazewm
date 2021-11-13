using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers
{
  public class ContainerService
  {
    /// <summary>
    /// The root node of the container tree. Monitors are the children of the root node, followed
    /// by workspaces, then split containers/windows.
    /// </summary>
    public Container ContainerTree = new Container();

    /// <summary>
    /// Containers (and their descendants) to redraw on the next invocation of
    /// `RedrawContainersCommand`.
    /// </summary>
    public List<Container> ContainersToRedraw = new List<Container>();

    /// <summary>
    /// The currently focused container. This can either be a `Window` or a
    /// `Workspace` without any descendant windows.
    /// </summary>
    public Container FocusedContainer => ContainerTree.LastFocusedDescendant;

    /// <summary>
    /// Whether a tiling or floating container is currently focused.
    /// </summary>
    public FocusMode FocusMode => FocusedContainer is FloatingWindow ?
      FocusMode.FLOATING : FocusMode.TILING;

    /// <summary>
    /// If set, this container overrides the target container to set focus to on the next
    /// focus window event (ie. `EVENT_SYSTEM_FOREGROUND`).
    /// </summary>
    public Container PendingFocusContainer = null;

    /// <summary>
    /// Whether the focused container of the WM is in sync with the OS. Mismatches between the
    /// focus state of the WM and the OS occur when ignored windows (eg. via user's window rules)
    /// or elevated windows are in focus.
    /// </summary>
    public bool IsFocusSynced
    {
      get
      {
        var foregroundHandle = GetForegroundWindow();
        var focusedContainer = FocusedContainer;

        if (focusedContainer is Window)
          return (focusedContainer as Window).Hwnd == foregroundHandle;

        else
          return IntPtr.Zero == foregroundHandle;
      }
    }

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
      var resizableSiblings = container.SelfAndSiblingsOfType(typeof(IResizable));

      return (int)((container as IResizable).SizePercentage * (parent.Width - (innerGap * (resizableSiblings.Count() - 1))));
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
      var resizableSiblings = container.SelfAndSiblingsOfType(typeof(IResizable));

      return (int)((container as IResizable).SizePercentage * (parent.Height - (innerGap * (resizableSiblings.Count() - 1))));
    }

    /// <summary>
    /// Calculates the X coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateXOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      var isFirstOfType = container.SelfAndSiblingsOfType(typeof(IResizable)).First() == container;

      if (parent.Layout == Layout.VERTICAL || isFirstOfType)
        return parent.X;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return container.GetPreviousSiblingOfType(typeof(IResizable)).X
        + container.GetPreviousSiblingOfType(typeof(IResizable)).Width
        + innerGap;
    }

    /// <summary>
    /// Calculates the Y coordinate of a container that can be resized programatically. This
    /// calculation is shared by windows and split containers.
    /// </summary>
    public int CalculateYOfResizableContainer(Container container)
    {
      var parent = container.Parent as SplitContainer;

      var isFirstOfType = container.SelfAndSiblingsOfType(typeof(IResizable)).First() == container;

      if (parent.Layout == Layout.HORIZONTAL || isFirstOfType)
        return parent.Y;

      var innerGap = _userConfigService.UserConfig.Gaps.InnerGap;

      return container.GetPreviousSiblingOfType(typeof(IResizable)).Y
        + container.GetPreviousSiblingOfType(typeof(IResizable)).Height
        + innerGap;
    }

    /// <summary>
    /// Traverse down a container in search of a descendant in the given direction. For example, get
    /// the rightmost container for `Direction.RIGHT`. Returns the `originContainer` if no suitable
    /// descendants are found. Any non-tiling containers are ignored.
    /// </summary>
    public Container GetDescendantInDirection(Container originContainer, Direction direction)
    {
      if (!(originContainer is SplitContainer) || !originContainer.HasChildren())
        return originContainer;

      var layout = (originContainer as SplitContainer).Layout;

      if (layout != direction.GetCorrespondingLayout())
        return GetDescendantInDirection(
          originContainer.LastFocusedChildOfType(typeof(IResizable)),
          direction
        );

      if (direction == Direction.UP || direction == Direction.LEFT)
        return GetDescendantInDirection(
          originContainer.ChildrenOfType(typeof(IResizable)).First(),
          direction
        );

      else
        return GetDescendantInDirection(
          originContainer.ChildrenOfType(typeof(IResizable)).Last(),
          direction
        );
    }

    /// <summary>
    /// Get the lowest container in the tree that has both `containerA` and `containerB` as
    /// descendants.
    /// </summary>
    public Container GetLowestCommonAncestor(Container containerA, Container containerB)
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
