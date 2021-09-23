using System.Linq;
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
    private WindowService _windowService;
    private UserConfigService _userConfigService;
    private ContainerService _containerService;

    public ResizeFocusedWindowHandler(Bus bus, WindowService windowService, UserConfigService userConfigService, ContainerService containerService)
    {
      _bus = bus;
      _windowService = windowService;
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

      // Ignore cases where the container to resize is a workspace or is only child.
      if (!containerToResize.HasSiblings() || containerToResize is Workspace)
        return CommandResponse.Ok;

      switch (resizeDirection)
      {
        case ResizeDirection.GROW_WIDTH:
        case ResizeDirection.GROW_HEIGHT:
          ShrinkSizeOfSiblings(containerToResize);
          break;

        case ResizeDirection.SHRINK_WIDTH:
        case ResizeDirection.SHRINK_HEIGHT:
          GrowSizeOfSiblings(containerToResize);
          break;
      }

      _containerService.SplitContainersToRedraw.Add(containerToResize.Parent as SplitContainer);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private void GrowSizeOfSiblings(Container containerToShrink)
    {
      var resizeProportion = _userConfigService.UserConfig.ResizeProportion;
      containerToShrink.SizePercentage -= resizeProportion;

      foreach (var sibling in containerToShrink.Siblings)
        sibling.SizePercentage += resizeProportion / containerToShrink.Siblings.Count();
    }

    private void ShrinkSizeOfSiblings(Container containerToGrow)
    {
      var resizeProportion = _userConfigService.UserConfig.ResizeProportion;
      containerToGrow.SizePercentage += resizeProportion;

      foreach (var sibling in containerToGrow.Siblings)
        sibling.SizePercentage -= resizeProportion / containerToGrow.Siblings.Count();
    }
  }
}
