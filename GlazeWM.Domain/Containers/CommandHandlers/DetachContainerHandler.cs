using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public DetachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DetachContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;
      var grandparent = parent.Parent;
      var siblings = childToRemove.Siblings;

      if (parent == null)
        throw new Exception("Cannot detach an already detached container. This is a bug.");

      parent.RemoveChild(childToRemove);

      var parentSiblings = parent.Siblings;
      var isEmptySplitContainer =
        !parent.HasChildren() && parent is SplitContainer and not Workspace;

      // Get the freed up space after container is detached.
      var availableSizePercentage = isEmptySplitContainer
        ? (parent as IResizable).SizePercentage
        : (childToRemove as IResizable)?.SizePercentage ?? 0;

      // Resize children of grandparent if `childToRemove`'s parent is also to be detached.
      var containersToResize = isEmptySplitContainer
        ? grandparent.ChildrenOfType<IResizable>()
        : parent.ChildrenOfType<IResizable>();

      // If the parent of the removed child is now an empty split container, detach the
      // split container as well.
      // TODO: Move out calls to `ContainersToRedraw.Add(...)`, since detaching might not
      // always require a redraw.
      if (isEmptySplitContainer)
      {
        _containerService.ContainersToRedraw.Add(parent.Parent);
        grandparent.RemoveChild(parent);
      }
      else
        _containerService.ContainersToRedraw.Add(parent);

      if (availableSizePercentage != 0)
      {
        var sizePercentageIncrement = availableSizePercentage / containersToResize.Count();

        // Adjust `SizePercentage` of the siblings of the removed container.
        foreach (var containerToResize in containersToResize)
          ((IResizable)containerToResize).SizePercentage += sizePercentageIncrement;
      }

      // var detachedSiblings = topMostDetached.Siblings;
      var detachedSiblings = isEmptySplitContainer ? parentSiblings : siblings;

      // If there is exactly *one* sibling to the detached container, then flatten that
      // sibling if it's a split container. This is to handle layouts like H[1 V[2 H[3]]],
      // where container 2 gets detached.
      if (detachedSiblings.Count() == 1 && detachedSiblings.ElementAt(0) is SplitContainer && childToRemove is not Workspace)
      {
        _bus.Invoke(
          new FlattenSplitContainerCommand(detachedSiblings.ElementAt(0) as SplitContainer)
        );

        // var topMostDetached = isEmptySplitContainer ? parent : childToRemove;
        var detachedParent = isEmptySplitContainer ? grandparent : parent;

        _bus.Invoke(
          new FlattenSplitContainerCommand(detachedParent as SplitContainer)
        );
      }

      return CommandResponse.Ok;
    }
  }
}
