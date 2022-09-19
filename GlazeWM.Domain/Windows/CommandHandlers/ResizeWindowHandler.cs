using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ResizeWindowHandler : ICommandHandler<ResizeWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private const double MIN_SIZE_PERCENTAGE = 0.01;

    public ResizeWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeWindowCommand command)
    {
      var dimensionToResize = command.DimensionToResize;
      var resizeAmount = command.ResizeAmount;
      var windowToResize = command.WindowToResize;

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

      // Get available size percentage amongst siblings.
      var availableSizePercentage = GetAvailableSizePercentage(
        resizableSiblings
      );

      // Prevent window from being smaller than the minimum and larger than space available from
      // sibling containers.
      var minResizeDelta = MIN_SIZE_PERCENTAGE - (containerToResize as IResizable).SizePercentage;
      var clampedResizePercentage = Math.Clamp(
        resizePercentage,
        minResizeDelta,
        availableSizePercentage
      );

      // Resize the container and distribute the size percentage amongst its siblings.
      (containerToResize as IResizable).SizePercentage += clampedResizePercentage;
      DistributeSizePercentage(resizableSiblings, clampedResizePercentage, availableSizePercentage);

      // TODO: Return early if `clampedResizePercentage` is 0 to avoid unnecessary redraws.
      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static double GetAvailableSizePercentage(IEnumerable<Container> containers)
    {
      return containers.Aggregate(
        0.0,
        (sum, container) => sum + (container as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE
      );
    }

    private static void DistributeSizePercentage(
      IEnumerable<Container> containers,
      double sizePercentage,
      double availableSizePercentage)
    {
      foreach (var container in containers)
      {
        var conAvailableSizePercentage =
          (container as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE;

        // Get percentage of resize that affects this container. `availableSizePercentage`
        // can be 0 here when the main container to resize is shrunk from max size percentage.
        var resizeFactor = availableSizePercentage == 0.0
          ? 1.0 / containers.Count()
          : conAvailableSizePercentage / availableSizePercentage;

        (container as IResizable).SizePercentage -= resizeFactor * sizePercentage;
      }
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
        (layout == Layout.HORIZONTAL && dimensionToResize == ResizeDimension.HEIGHT) ||
        (layout == Layout.VERTICAL && dimensionToResize == ResizeDimension.WIDTH);

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
          dimensionToResize == ResizeDimension.WIDTH
            ? sum + container.Width
            : sum + container.Height
      );

      return 1.0 / resizableLength;
    }
  }
}
