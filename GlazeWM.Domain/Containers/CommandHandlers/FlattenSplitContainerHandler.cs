using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class FlattenSplitContainerHandler : ICommandHandler<FlattenSplitContainerCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public FlattenSplitContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
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
        // Remove the child from the split container.
        splitContainer.Children.Remove(child);
        splitContainer.ChildFocusOrder.Remove(child);

        // Insert child at its original index in the parent.
        parent.Children.Insert(index + childIndex, child);
        child.Parent = parent;

        if (child is IResizable childResizable)
          childResizable.SizePercentage = splitContainer.SizePercentage * childResizable.SizePercentage;

        // Inverse the tiling direction of any child split containers.
        if (child is SplitContainer childSplitContainer)
          childSplitContainer.TilingDirection = childSplitContainer.TilingDirection.Inverse();
      }

      // Remove the split container from the tree.
      parent.Children.Remove(splitContainer);
      splitContainer.Parent = null;

      // Correct focus order of the inserted containers.
      foreach (var child in children)
      {
        var childFocusIndex = focusOrder.IndexOf(child);
        parent.ChildFocusOrder.ShiftToIndex(focusIndex + childFocusIndex, child);
      }

      // TODO: Remove unnecessary redraws.
      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
