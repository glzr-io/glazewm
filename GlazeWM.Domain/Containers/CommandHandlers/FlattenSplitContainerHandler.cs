using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class FlattenSplitContainerHandler : ICommandHandler<FlattenSplitContainerCommand>
  {
    private readonly ContainerService _containerService;

    public FlattenSplitContainerHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(FlattenSplitContainerCommand command)
    {
      var splitContainer = command.ContainerToFlatten;

      // Keep references to properties of container to flatten prior to detaching.
      var parent = splitContainer.Parent;
      var index = splitContainer.Index;
      var focusIndex = splitContainer.FocusIndex;
      var children = splitContainer.Children.ToList();
      var focusOrder = splitContainer.ChildFocusOrder.ToList();

      foreach (var (child, childIndex) in children.WithIndex())
      {
        // Insert child at its original index in the parent.
        splitContainer.RemoveChild(child);
        parent.Children.Insert(index + childIndex, child);
        child.Parent = parent;

        if (child is IResizable childResizable)
          childResizable.SizePercentage = splitContainer.SizePercentage * childResizable.SizePercentage;

        // Inverse the tiling direction of any child split containers.
        if (child is SplitContainer childSplitContainer)
          childSplitContainer.TilingDirection = childSplitContainer.TilingDirection.Inverse();
      }

      // Remove the split container from the tree.
      parent.RemoveChild(splitContainer);

      // Correct focus order of the inserted containers.
      parent.ChildFocusOrder.InsertRange(focusIndex, focusOrder);

      // TODO: Remove unnecessary redraws.
      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
