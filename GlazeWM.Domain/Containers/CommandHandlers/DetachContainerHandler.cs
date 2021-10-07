using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public DetachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DetachContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;

      parent.RemoveChild(childToRemove);
      parent.ChildFocusOrder.Remove(childToRemove);

      if (childToRemove is TilingWindow || childToRemove is SplitContainer)
        AdjustSiblingSizes(parent);

      return CommandResponse.Ok;
    }

    private void AdjustSiblingSizes(Container parent)
    {
      // Siblings of the removed child.
      var siblings = parent.Children;

      var isEmptySplitContainer = parent is SplitContainer && !parent.HasChildren()
        && !(parent is Workspace);

      // If the parent of the removed child is an empty split container, remove
      // the split container as well.
      if (isEmptySplitContainer)
      {
        _bus.Invoke(new DetachContainerCommand(parent));
        return;
      }

      // TODO: Adjust SizePercentage of children based on their previous SizePercentage.

      var resizableSiblings = siblings.Where(container => container is IResizable);
      double defaultPercent = 1.0 / resizableSiblings.Count();

      // Adjust `SizePercentage` of the siblings of the removed container.
      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent as SplitContainer);
    }
  }
}
