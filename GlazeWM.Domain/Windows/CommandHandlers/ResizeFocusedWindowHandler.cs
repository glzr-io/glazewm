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

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ResizeFocusedWindowHandler : ICommandHandler<ResizeFocusedWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ResizeFocusedWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeFocusedWindowCommand command)
    {
      var dimensionToResize = command.DimensionToResize;
      var resizeAmount = command.ResizeAmount;

      // Ignore cases where focused container is not a tiling window.
      if (_containerService.FocusedContainer is not TilingWindow focusedWindow)
        return CommandResponse.Ok;

      var layout = (focusedWindow.Parent as SplitContainer).Layout;

      // Get whether the parent of the focused window should be resized rather than the focused
      // window itself.
      var shouldResizeParent =
        (layout == Layout.HORIZONTAL && dimensionToResize == ResizeDimension.HEIGHT)
        || (layout == Layout.VERTICAL && dimensionToResize == ResizeDimension.WIDTH);

      var containerToResize = shouldResizeParent ? focusedWindow.Parent : focusedWindow;

      // Get siblings that can be resized.
      var resizableSiblings = containerToResize.Siblings.Where(container => container is IResizable);

      // Ignore cases where the container to resize is a workspace or is only child.
      if (!resizableSiblings.Any() || containerToResize is Workspace)
        return CommandResponse.Ok;

      // Convert `resizeAmount` to a percentage to increase/decrease the window size by.
      var resizePercentage = ConvertToResizePercentage(resizeAmount);
      var resizeProportion = resizePercentage / 100;

      (containerToResize as IResizable).SizePercentage += resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage -= resizeProportion / resizableSiblings.Count();

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      Bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static double ConvertToResizePercentage(string resizeAmount)
    {
      try
      {
        var matchedResizeAmount = new Regex(@"(.*)(%|ppt)").Match(resizeAmount);
        var amount = matchedResizeAmount.Groups[1].Value;
        var unit = matchedResizeAmount.Groups[2].Value;

        // TODO: Handle conversion from px to `SizePercentage`.
        return unit switch
        {
          "%" => Convert.ToDouble(amount, CultureInfo.InvariantCulture),
          "ppt" => Convert.ToDouble(amount, CultureInfo.InvariantCulture),
          _ => throw new ArgumentException(nameof(unit)),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid resize amount {resizeAmount}.");
      }
    }
  }
}
