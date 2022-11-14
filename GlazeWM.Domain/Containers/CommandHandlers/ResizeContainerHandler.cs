using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class ResizeContainerHandler : ICommandHandler<ResizeContainerCommand>
  {
    private const double MIN_SIZE_PERCENTAGE = 0.01;

    public CommandResponse Handle(ResizeContainerCommand command)
    {
      var containerToResize = command.ContainerToResize;
      var resizePercentage = command.ResizePercentage;

      var resizableSiblings = containerToResize.SiblingsOfType<IResizable>();

      // Ignore cases where the container to resize is a workspace or the only child.
      if (!resizableSiblings.Any() || containerToResize is Workspace)
        return CommandResponse.Ok;

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

      // Resize the container.
      (containerToResize as IResizable).SizePercentage += clampedResizePercentage;

      // Distribute the size percentage amongst its siblings.
      foreach (var sibling in resizableSiblings)
      {
        (sibling as IResizable).SizePercentage -= GetSiblingResizePercentage(
          sibling,
          resizableSiblings.Count(),
          clampedResizePercentage,
          availableSizePercentage
        );
      }

      return CommandResponse.Ok;
    }

    private static double GetAvailableSizePercentage(IEnumerable<Container> containers)
    {
      return containers.Aggregate(
        0.0,
        (sum, container) => sum + (container as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE
      );
    }

    private static double GetSiblingResizePercentage(
      Container siblingToResize,
      int siblingCount,
      double sizePercentage,
      double availableSizePercentage)
    {
      var conAvailableSizePercentage =
        (siblingToResize as IResizable).SizePercentage - MIN_SIZE_PERCENTAGE;

      // Get percentage of resize that affects this container. `availableSizePercentage`
      // can be 0 here when the main container to resize is shrunk from max size percentage.
      var resizeFactor = availableSizePercentage == 0.0 || sizePercentage < 0
        ? 1.0 / siblingCount
        : conAvailableSizePercentage / availableSizePercentage;

      return resizeFactor * sizePercentage;
    }
  }
}
