using System.Linq;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
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
      var parent = command.Parent;
      var childToRemove = command.ChildToRemove;

      parent.ChildFocusOrder.Remove(childToRemove);
      parent.RemoveChild(childToRemove);

      AdjustSiblingSizes(parent);

      return CommandResponse.Ok;
    }

    private void AdjustSiblingSizes(SplitContainer parent)
    {
      // Siblings of the removed child.
      var siblings = parent.Children;

      var isEmptySplitContainer = !parent.HasChildren() && !(parent is Workspace);

      // If the parent of the removed child is an empty split container, remove
      // the split container as well.
      if (isEmptySplitContainer)
      {
        var grandparent = parent.Parent;
        grandparent.RemoveChild(parent);
        grandparent.ChildFocusOrder.Remove(parent);

        // TODO: Perhaps create a private method that takes the container with children
        // to adjust that has the SizePercentage and default percent logic. Alternatively
        // create a variable containerToAdjust that is then operated on.
        foreach (var child in grandparent.Children)
          child.SizePercentage = 1.0 / grandparent.Children.Count;

        _containerService.SplitContainersToRedraw.Add(grandparent as SplitContainer);
        return;
      }

      // TODO: Adjust SizePercentage of children based on their previous SizePercentage.

      foreach (var child in siblings)
        child.SizePercentage = 1.0 / siblings.Count;

      _containerService.SplitContainersToRedraw.Add(parent);
    }
  }
}
