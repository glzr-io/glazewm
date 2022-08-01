using System;
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

      var layout = (windowToResize.Parent as SplitContainer).Layout;

      // Get whether the parent of the window should be resized rather than the window itself.
      var shouldResizeParent =
        (layout == Layout.HORIZONTAL && dimensionToResize == ResizeDimension.HEIGHT)
        || (layout == Layout.VERTICAL && dimensionToResize == ResizeDimension.WIDTH);

      // Get container and its siblings to resize.
      var containerToResize = shouldResizeParent ? windowToResize.Parent : windowToResize;
      var resizableSiblings = containerToResize.SiblingsOfType(typeof(IResizable));

      // Ignore cases where the container to resize is a workspace or is the only child.
      if (!resizableSiblings.Any() || containerToResize is Workspace)
        return CommandResponse.Ok;

      // Convert `resizeAmount` to a percentage to increase/decrease the window size by.
      var pixelScaleFactor = GetPixelScaleFactor(containerToResize, dimensionToResize);
      var resizeProportion = ConvertToResizePercentage(resizeAmount, pixelScaleFactor);

      (containerToResize as IResizable).SizePercentage += resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage -= resizeProportion / resizableSiblings.Count();

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static double GetPixelScaleFactor(
      Container containerToResize,
      ResizeDimension dimensionToResize
    )
    {
      // Get area that can be resized (ie. exclude inner gaps).
      var resizableArea = containerToResize.SelfAndSiblings.Aggregate(1.0, (acc, con) =>
      {
        return dimensionToResize == ResizeDimension.WIDTH
          ? acc + con.Width
          : acc + con.Height;
      });

      return 1.0 / resizableArea;
    }

    private static double ConvertToResizePercentage(string resizeAmount, double pixelScaleFactor)
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
          "px" => floatAmount * pixelScaleFactor,
          _ => throw new ArgumentException(null, nameof(resizeAmount)),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid resize amount {resizeAmount}.");
      }
    }
  }
}
