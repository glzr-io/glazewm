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
      var resizableSiblings = containerToResize.SiblingsOfType(typeof(IResizable));

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
      var distributableSizePercentage = GetDistributableSizePercentage(
        resizableSiblings
      );

      // Prevent window from being smaller than the minimum and larger than space available from
      // sibling containers.
      var clampedResizePercentage = Math.Clamp(
        resizePercentage,
        MIN_SIZE_PERCENTAGE - (containerToResize as IResizable).SizePercentage,
        distributableSizePercentage
      );

      if (clampedResizePercentage == 0 || distributableSizePercentage == 0)
        return CommandResponse.Ok;

      // Resize the container and distribute the size percentage amongst its siblings.
      (containerToResize as IResizable).SizePercentage += clampedResizePercentage;
      DistributeSizePercentage(resizableSiblings, clampedResizePercentage);

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static double GetDistributableSizePercentage(IEnumerable<Container> containers)
    {
      return containers.Aggregate(
        0.0,
        (sum, container) => sum + (container as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE
      );
    }

    private static void DistributeSizePercentage(
      IEnumerable<Container> containers,
      double sizePercentage
    )
    {
      var distributableSizePercentage = GetDistributableSizePercentage(
        containers
      );

      foreach (var container in containers)
      {
        // Get percentage of resize that affects this container.
        var resizeFactor =
          ((container as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE) /
          distributableSizePercentage;

        (container as IResizable).SizePercentage -= resizeFactor * sizePercentage;
      }
    }

    private static Container GetContainerToResize(
      Window windowToResize,
      ResizeDimension dimensionToResize
    )
    {
      var parent = windowToResize.Parent;
      var grandparent = parent.Parent;
      var layout = (parent as SplitContainer).Layout;

      // Whether the resize is in the inverse direction of its layout.
      var isInverseResize =
        (layout == Layout.HORIZONTAL && dimensionToResize == ResizeDimension.HEIGHT) ||
        (layout == Layout.VERTICAL && dimensionToResize == ResizeDimension.WIDTH);

      if (!isInverseResize && !windowToResize.Siblings.Any() && grandparent is IResizable)
        return grandparent;

      return isInverseResize ? parent : windowToResize;
    }

    private static double ConvertToResizePercentage(
      Container containerToResize,
      ResizeDimension dimensionToResize,
      string resizeAmount
      )
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
      ResizeDimension dimensionToResize
    )
    {
      // Get available width/height that can be resized (ie. exclude inner gaps).
      var resizableLength = containerToResize.SelfAndSiblings.Aggregate(
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
