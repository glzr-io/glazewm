using System.Collections.Generic;
using System.Linq;
using System.Windows.Documents;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ResizeFocusedWindowHandler : ICommandHandler<ResizeFocusedWindowCommand>
  {
    private Bus _bus;
    private UserConfigService _userConfigService;
    private ContainerService _containerService;

    public ResizeFocusedWindowHandler(Bus bus, UserConfigService userConfigService, ContainerService containerService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var layout = (focusedWindow.Parent as SplitContainer).Layout;
      var resizeDirection = command.ResizeDirection;

      // Whether the parent of the focused window should be resized rather than the focused window itself.
      var shouldResizeParent =
        (layout == Layout.HORIZONTAL &&
          (resizeDirection == ResizeDirection.SHRINK_HEIGHT || resizeDirection == ResizeDirection.GROW_HEIGHT)) ||
        (layout == Layout.VERTICAL &&
          (resizeDirection == ResizeDirection.SHRINK_WIDTH || resizeDirection == ResizeDirection.GROW_WIDTH));

      var containerToResize = shouldResizeParent ? focusedWindow.Parent : focusedWindow;

      // Get siblings that can be resized.
      var resizableSiblings = containerToResize.Siblings.Where(container => container is IResizable);

      // Ignore cases where the container to resize is a workspace or is only child.
      if (resizableSiblings.Count() == 0 || containerToResize is Workspace)
        return CommandResponse.Ok;

      switch (resizeDirection)
      {
        case ResizeDirection.GROW_WIDTH:
        case ResizeDirection.GROW_HEIGHT:
          ShrinkSizeOfSiblings(containerToResize, resizableSiblings);
          break;

        case ResizeDirection.SHRINK_WIDTH:
        case ResizeDirection.SHRINK_HEIGHT:
          GrowSizeOfSiblings(containerToResize, resizableSiblings);
          break;
      }

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private void GrowSizeOfSiblings(Container containerToShrink, IEnumerable<Container> resizableSiblings)
    {
      var resizeProportion = _userConfigService.UserConfig.ResizeProportion;
      (containerToShrink as IResizable).SizePercentage -= resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage += resizeProportion / resizableSiblings.Count();
    }

    private void ShrinkSizeOfSiblings(Container containerToGrow, IEnumerable<Container> resizableSiblings)
    {
      var resizeProportion = _userConfigService.UserConfig.ResizeProportion;
      (containerToGrow as IResizable).SizePercentage += resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage -= resizeProportion / resizableSiblings.Count();
    }
  }
}
