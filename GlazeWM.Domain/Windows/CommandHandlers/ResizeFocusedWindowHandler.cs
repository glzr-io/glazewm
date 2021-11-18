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
    private Bus _bus;
    private ContainerService _containerService;

    public ResizeFocusedWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ResizeFocusedWindowCommand command)
    {
      var resizeDirection = command.ResizeDirection;
      var resizeAmount = command.ResizeAmount;
      var focusedWindow = _containerService.FocusedContainer as TilingWindow;

      // Ignore cases where focused container is not a tiling window.
      if (focusedWindow == null)
        return CommandResponse.Ok;

      var layout = (focusedWindow.Parent as SplitContainer).Layout;

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

      // Convert `resizeAmount` to a percentage to increase/decrease the window size by.
      var resizePercentage = ConvertToResizePercentage(resizeAmount);

      switch (resizeDirection)
      {
        case ResizeDirection.GROW_WIDTH:
        case ResizeDirection.GROW_HEIGHT:
          ShrinkSizeOfSiblings(containerToResize, resizableSiblings, resizePercentage);
          break;

        case ResizeDirection.SHRINK_WIDTH:
        case ResizeDirection.SHRINK_HEIGHT:
          GrowSizeOfSiblings(containerToResize, resizableSiblings, resizePercentage);
          break;
      }

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private double ConvertToResizePercentage(string resizeAmount)
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
          _ => throw new ArgumentException(),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid resize amount {resizeAmount}.");
      }
    }

    private void GrowSizeOfSiblings(
      Container containerToShrink,
      IEnumerable<Container> resizableSiblings,
      double resizePercentage
    )
    {
      var resizeProportion = resizePercentage / 100;
      (containerToShrink as IResizable).SizePercentage -= resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage += resizeProportion / resizableSiblings.Count();
    }

    private void ShrinkSizeOfSiblings(
      Container containerToGrow,
      IEnumerable<Container> resizableSiblings,
      double resizePercentage
    )
    {
      var resizeProportion = resizePercentage / 100;
      (containerToGrow as IResizable).SizePercentage += resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage -= resizeProportion / resizableSiblings.Count();
    }
  }
}
