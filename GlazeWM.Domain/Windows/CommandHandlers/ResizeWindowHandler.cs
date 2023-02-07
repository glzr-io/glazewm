using System;
using System.Globalization;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class ResizeWindowHandler : ICommandHandler<ResizeWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly MonitorService _monitorService;

    public ResizeWindowHandler(Bus bus, ContainerService containerService, MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(ResizeWindowCommand command)
    {
      var dimensionToResize = command.DimensionToResize;
      var resizeAmount = command.ResizeAmount;
      var windowToResize = command.WindowToResize;

      if (windowToResize is FloatingWindow)
      {
        ResizeFloatingWindow(windowToResize, dimensionToResize, resizeAmount);
        return CommandResponse.Ok;
      }
      // Ignore cases where window is not tiling.
      if (windowToResize is not TilingWindow)
        return CommandResponse.Ok;

      // Get container and its siblings to resize.
      var containerToResize = GetContainerToResize(windowToResize, dimensionToResize);
      var resizableSiblings = containerToResize.SiblingsOfType<IResizable>();

      // Ignore cases where the container to resize is a workspace or the only child.
      if (!resizableSiblings.Any() || containerToResize is Workspace)
        return CommandResponse.Ok;

      // Convert `resizeAmount` to a percentage to increase/decrease the window size by.
      var resizePercentage = ConvertToResizePercentage(
        containerToResize,
        dimensionToResize,
        resizeAmount
      );

      _bus.Invoke(new ResizeContainerCommand(containerToResize, resizePercentage));

      // TODO: Return early if `clampedResizePercentage` is 0 to avoid unnecessary redraws.
      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static Container GetContainerToResize(
      Window windowToResize,
      ResizeDimension dimensionToResize)
    {
      var parent = windowToResize.Parent;
      var grandparent = parent.Parent;
      var layout = (parent as SplitContainer).Layout;

      // Whether the resize is in the inverse direction of its layout.
      var isInverseResize =
        (layout == Layout.Horizontal && dimensionToResize == ResizeDimension.Height) ||
        (layout == Layout.Vertical && dimensionToResize == ResizeDimension.Width);

      var hasResizableSiblings = windowToResize.SiblingsOfType<IResizable>().Any();

      if (!isInverseResize && !hasResizableSiblings && grandparent is IResizable)
        return grandparent;

      return isInverseResize ? parent : windowToResize;
    }

    private static double ConvertToResizePercentage(
      Container containerToResize,
      ResizeDimension dimensionToResize,
      string resizeAmount)
    {
      try
      {
        var matchedResizeAmount = new Regex("(.*)(%|ppt|px)").Match(resizeAmount);
        var amount = matchedResizeAmount.Groups[1].Value;
        var unit = matchedResizeAmount.Groups[2].Value;
        var floatAmount = Convert.ToDouble(amount, CultureInfo.InvariantCulture);

        return unit switch
        {
          "%" => floatAmount / 100,
          "ppt" => floatAmount / 100,
          "px" => floatAmount * GetPixelScaleFactor(containerToResize, dimensionToResize),
          _ => throw new ArgumentException(null, nameof(resizeAmount)),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid resize amount {resizeAmount}.");
      }
    }

    private static double GetPixelScaleFactor(
      Container containerToResize,
      ResizeDimension dimensionToResize)
    {
      // Get available width/height that can be resized (ie. exclude inner gaps).
      var resizableLength = containerToResize.SelfAndSiblingsOfType<IResizable>().Aggregate(
        1.0,
        (sum, container) =>
          dimensionToResize == ResizeDimension.Width
            ? sum + container.Width
            : sum + container.Height
      );

      return 1.0 / resizableLength;
    }
    private void ResizeFloatingWindow(Window windowToResize, ResizeDimension dimensionToResize, string resizeAmount)
    {
      const int MIN_WIDTH = 250;
      const int MIN_HEIGHT = 140;
      var resizePercentage = ConvertToResizePercentage(windowToResize, dimensionToResize, resizeAmount);
      int amount = (int)(windowToResize.Parent.Width * resizePercentage);

      var width = windowToResize.FloatingPlacement.Width;
      var height = windowToResize.FloatingPlacement.Height;

      _ = dimensionToResize switch
      {
        ResizeDimension.Width => width += amount,
        ResizeDimension.Height => height += amount,
        _ => throw new ArgumentException(null, nameof(dimensionToResize))
      };

      //Return if resize gonna make window smaller than allowed
      //Allow increasing size (for situations if user made the window smaller
      //  than MIN_WIDHT or MIN_HEIGHT with the mouse)
      if ((width < MIN_WIDTH || height < MIN_HEIGHT) && amount < 0)
        return;

      windowToResize.FloatingPlacement = Rect.FromXYCoordinates(windowToResize.FloatingPlacement.X, windowToResize.FloatingPlacement.Y, width, height);

      _containerService.ContainersToRedraw.Add(windowToResize);
      _bus.Invoke(new RedrawContainersCommand());

      // Check if window now takes up more of another screen
      var currentWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToResize);

      // Get workspace that encompasses most of the window.
      var targetMonitor = _monitorService.GetMonitorFromHandleLocation(windowToResize.Handle);
      var targetWorkspace = targetMonitor.DisplayedWorkspace;

      // Ignore if window is still within the bounds of its current workspace.
      if (currentWorkspace == targetWorkspace)
      {
        return;
      }

      // Change the window's parent workspace.
      _bus.Invoke(new MoveContainerWithinTreeCommand(windowToResize, targetWorkspace, false));
      _bus.Emit(new FocusChangedEvent(windowToResize));

      // Redrawing again to fix weird WindowsOS behaviour
      _containerService.ContainersToRedraw.Add(windowToResize);
      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
