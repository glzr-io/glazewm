using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
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

    public dynamic Handle(ResizeFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      // Ignore cases where focused container is not a window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var siblings = focusedWindow.Siblings;

      // Ignore cases where focused window doesn't have any siblings.
      if (siblings.Count() == 0)
        return CommandResponse.Ok;

      var parent = focusedWindow.Parent as SplitContainer;
      var layout = parent.Layout;
      var resizeDirection = command.ResizeDirection;

      if (
        layout == Layout.Horizontal && resizeDirection == ResizeDirection.GROW_WIDTH
        || layout == Layout.Vertical && resizeDirection == ResizeDirection.GROW_HEIGHT
      )
      {
        if (focusedWindow.Siblings.Count() == 0)
          return CommandResponse.Ok;

        DecreaseSiblingSizes(focusedWindow);
        _containerService.SplitContainersToRedraw.Add(parent);
      }

      if (
        layout == Layout.Vertical && resizeDirection == ResizeDirection.GROW_WIDTH
        || layout == Layout.Horizontal && resizeDirection == ResizeDirection.GROW_HEIGHT
      )
      {
        var containerToResize = focusedWindow.Parent;
        if (containerToResize.Siblings.Count() == 0 || containerToResize is Workspace)
          return CommandResponse.Ok;

        DecreaseSiblingSizes(containerToResize);
        _containerService.SplitContainersToRedraw.Add(containerToResize.Parent as SplitContainer);
      }

      if (
        layout == Layout.Horizontal && resizeDirection == ResizeDirection.SHRINK_WIDTH
        || layout == Layout.Vertical && resizeDirection == ResizeDirection.SHRINK_HEIGHT
      )
      {
        if (focusedWindow.Siblings.Count() == 0)
          return CommandResponse.Ok;

        IncreaseSiblingSizes(focusedWindow);
        _containerService.SplitContainersToRedraw.Add(parent);
      }

      if (
        layout == Layout.Vertical && resizeDirection == ResizeDirection.SHRINK_WIDTH
        || layout == Layout.Horizontal && resizeDirection == ResizeDirection.SHRINK_HEIGHT
      )
      {
        var containerToResize = focusedWindow.Parent;
        if (containerToResize.Siblings.Count() == 0 || containerToResize is Workspace)
          return CommandResponse.Ok;

        IncreaseSiblingSizes(containerToResize);
        _containerService.SplitContainersToRedraw.Add(containerToResize.Parent as SplitContainer);
      }

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private void IncreaseSiblingSizes(Container containerToShrink)
    {
      var resizePercentage = _userConfigService.UserConfig.ResizePercentage;
      containerToShrink.SizePercentage -= resizePercentage;

      foreach (var sibling in containerToShrink.Siblings)
        sibling.SizePercentage += resizePercentage / containerToShrink.Siblings.Count();
    }

    private void DecreaseSiblingSizes(Container containerToGrow)
    {
      var resizePercentage = _userConfigService.UserConfig.ResizePercentage;
      containerToGrow.SizePercentage += resizePercentage;

      foreach (var sibling in containerToGrow.Siblings)
        sibling.SizePercentage -= resizePercentage / containerToGrow.Siblings.Count();
    }
  }
}
