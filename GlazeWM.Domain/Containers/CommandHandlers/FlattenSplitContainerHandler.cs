using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
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
      var containerToFlatten = command.ContainerToFlatten;

      // Keep references to properties of container to flatten prior to detaching.
      // DetachContainerCommand will flatten all SplitContainers with a single child, so use the properties
      // of the outermost split container matching that criteria
      var outermostSplit = containerToFlatten.SelfAndAncestors
        .TakeWhile(ancestor => ancestor is SplitContainer and not Workspace && ancestor.Children.Count == 1)
        .Last();

      var originalParent = outermostSplit.Parent;
      var originalChildren = outermostSplit.Children.ToList();
      var originalFocusIndex = outermostSplit.FocusIndex;
      var originalIndex = outermostSplit.Index;
      var originalFocusOrder = outermostSplit.ChildFocusOrder.ToList();

      foreach (var (child, index) in originalChildren.WithIndex())
      {
        // Insert children of the split container at its original index in the parent. The split
        // container will automatically detach once its last child is detached.
        _bus.Invoke(new DetachContainerCommand(child));
        _bus.Invoke(new AttachContainerCommand(child, originalParent, originalIndex + index));

        (child as IResizable).SizePercentage *= (outermostSplit as IResizable).SizePercentage;
      }

      // Correct focus order of the inserted containers.
      foreach (var child in originalChildren)
      {
        var childFocusIndex = originalFocusOrder.IndexOf(child);
        originalParent.ChildFocusOrder.ShiftToIndex(originalFocusIndex + childFocusIndex, child);
      }

      _containerService.ContainersToRedraw.Add(originalParent);

      return CommandResponse.Ok;
    }
  }
}
