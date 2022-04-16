using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class DetachAndResizeContainerHandler : ICommandHandler<DetachAndResizeContainerCommand>
  {
    private readonly Bus _bus;

    public DetachAndResizeContainerHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(DetachAndResizeContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;
      var grandparent = parent.Parent;

      if (childToRemove is not IResizable)
        throw new Exception("Cannot resize a non-resizable container. This is a bug.");

      var isEmptySplitContainer = parent is SplitContainer && parent.Children.Count == 1
        && parent is not Workspace;

      // Get the freed up space after container is detached.
      var availableSizePercentage = isEmptySplitContainer
        ? (parent as IResizable).SizePercentage
        : (childToRemove as IResizable).SizePercentage;

      // Resize children of grandparent if `childToRemove`'s parent is also to be detached.
      var containersToResize = isEmptySplitContainer
        ? grandparent.Children.Where(container => container is IResizable)
        : parent.Children.Where(container => container is IResizable);

      Bus.Invoke(new DetachContainerCommand(childToRemove));

      var sizePercentageIncrement = availableSizePercentage / containersToResize.Count();

      // Adjust `SizePercentage` of the siblings of the removed container.
      foreach (var containerToResize in containersToResize)
      {
        ((IResizable)containerToResize).SizePercentage += sizePercentageIncrement;
      }

      return CommandResponse.Ok;
    }
  }
}
