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

      var containerToResize = shouldResizeParent ? windowToResize.Parent : windowToResize;

      // Get siblings that can be resized.
      var resizableSiblings = containerToResize.Siblings.Where(container => container is IResizable);

      // Ignore cases where the container to resize is a workspace or is only child.
      if (!resizableSiblings.Any() || containerToResize is Workspace)
        return CommandResponse.Ok;

      // Convert `resizeAmount` to a percentage to increase/decrease the window size by.
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(windowToResize);
      var resizeProportion = ConvertToResizeProportion(resizeAmount, workspace, dimensionToResize);

      (containerToResize as IResizable).SizePercentage += resizeProportion;

      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage -= resizeProportion / resizableSiblings.Count();

      _containerService.ContainersToRedraw.Add(containerToResize.Parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static double ConvertToResizeProportion(
      string resizeAmount,
      Workspace parentWorkspace,
      ResizeDimension dimensionToResize
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
          "px" => dimensionToResize == ResizeDimension.WIDTH
            ? floatAmount / parentWorkspace.Width
            : floatAmount / parentWorkspace.Height,
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
