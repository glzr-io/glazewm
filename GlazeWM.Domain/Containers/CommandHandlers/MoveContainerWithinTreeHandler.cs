using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class MoveContainerWithinTreeHandler : ICommandHandler<MoveContainerWithinTreeCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public MoveContainerWithinTreeHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(MoveContainerWithinTreeCommand command)
    {
      var containerToMove = command.ContainerToMove;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      // Get lowest common ancestor (LCA) between `containerToMove` and `targetParent`. This could
      // be the `targetParent` itself.
      var lowestCommonAncestor = ContainerService.GetLowestCommonAncestor(
        containerToMove,
        targetParent
      );

      var focusedContainer = _containerService.FocusedContainer;

      // Handle case where target parent is the LCA (eg. in the case of swapping sibling containers
      // or moving a container to a direct ancestor).
      if (targetParent == lowestCommonAncestor)
      {
        MoveToLowestCommonAncestor(
          containerToMove,
          lowestCommonAncestor,
          targetIndex
        );

        if (containerToMove == focusedContainer)
          _bus.Emit(new FocusedContainerMovedEvent(containerToMove));

        return CommandResponse.Ok;
      }

      // Get ancestors of `containerToMove` and `targetParent` that are direct children of the LCA.
      // This could be the `containerToMove` or `targetParent` itself.
      var containerToMoveAncestor = containerToMove.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);

      // Get whether the container is the focused descendant in its original subtree.
      var isFocusedDescendant = containerToMove == containerToMoveAncestor
        || containerToMoveAncestor.LastFocusedDescendant.SelfAndAncestors.Contains(containerToMove);

      var targetParentAncestor = targetParent.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);

      // Get whether the ancestor of `containerToMove` appears before `targetParent`'s ancestor in
      // the `ChildFocusOrder` of LCA.
      var originalFocusIndex = containerToMoveAncestor.FocusIndex;
      var isSubtreeFocused = originalFocusIndex < targetParentAncestor.FocusIndex;

      _bus.Invoke(new DetachContainerCommand(containerToMove));
      _bus.Invoke(new AttachContainerCommand(containerToMove, targetParent, targetIndex));

      // Set `containerToMove` as focused descendant within target subtree if its original subtree
      // had focus more recently (even if the container is not the last focused within that subtree).
      if (isSubtreeFocused)
        _bus.Invoke(new SetFocusedDescendantCommand(containerToMove, targetParentAncestor));

      // If the focused descendant is moved to the targets subtree, then the target's ancestor
      // should be placed before the original ancestor in LCA's `ChildFocusOrder`.
      if (isFocusedDescendant && isSubtreeFocused)
        lowestCommonAncestor.ChildFocusOrder.ShiftToIndex(
          originalFocusIndex,
          targetParentAncestor
        );

      if (containerToMove == focusedContainer)
        _bus.Emit(new FocusedContainerMovedEvent(containerToMove));

      return CommandResponse.Ok;
    }

    private void MoveToLowestCommonAncestor(
      Container containerToMove,
      Container lowestCommonAncestor,
      int targetIndex)
    {
      // Keep reference to focus index of container's ancestor in LCA's `ChildFocusOrder`.
      var originalFocusIndex = containerToMove.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor)
        .FocusIndex;

      // Keep reference to container index and number of children that LCA has.
      var originalIndex = containerToMove.Index;
      var originalLcaChildCount = lowestCommonAncestor.Children.Count;

      _bus.Invoke(new DetachContainerCommand(containerToMove));

      var newLcaChildCount = lowestCommonAncestor.Children.Count;
      var shouldAdjustTargetIndex = originalLcaChildCount > newLcaChildCount
        && originalIndex < targetIndex;

      // Adjust for when target index changes on detach of container. For example, when shifting a
      // top-level container to the right in a workspace.
      var adjustedTargetIndex = shouldAdjustTargetIndex ? targetIndex - 1 : targetIndex;

      _bus.Invoke(
        new AttachContainerCommand(
          containerToMove,
          lowestCommonAncestor,
          adjustedTargetIndex
        )
      );

      lowestCommonAncestor.ChildFocusOrder.ShiftToIndex(originalFocusIndex, containerToMove);
    }
  }
}
