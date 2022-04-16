using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class MoveContainerWithinTreeHandler : ICommandHandler<MoveContainerWithinTreeCommand>
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
      var shouldAdjustSize = command.ShouldAdjustSize;

      if (shouldAdjustSize && containerToMove is not IResizable)
        throw new Exception("Cannot resize a non-resizable container. This is a bug.");

      // Get lowest common ancestor (LCA) between `containerToMove` and `targetParent`. This could
      // be the `targetParent` itself.
      var lowestCommonAncestor = ContainerService.GetLowestCommonAncestor(
        containerToMove,
        targetParent
      );

      // Handle case where target parent is the LCA (eg. in the case of swapping sibling containers
      // or moving a container to a direct ancestor).
      if (targetParent == lowestCommonAncestor)
      {
        MoveToLowestCommonAncestor(containerToMove, lowestCommonAncestor, targetIndex, shouldAdjustSize);
        return CommandResponse.Ok;
      }

      // Get ancestors of `containerToMove` and `targetParent` that are direct children of the LCA.
      // This could be the `containerToMove` or `targetParent` itself.
      var containerToMoveAncestor = containerToMove.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);

      // Get whether the container is the focused descendant in its original subtree.
      var isFocusedDescendant = containerToMove == containerToMoveAncestor
        || containerToMoveAncestor.LastFocusedDescendant == containerToMove;

      var targetParentAncestor = targetParent.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);

      // Get whether the ancestor of `containerToMove` appears before `targetParent`'s ancestor in
      // the `ChildFocusOrder` of LCA.
      var originalFocusIndex = containerToMoveAncestor.FocusIndex;
      var isSubtreeFocused = originalFocusIndex < targetParentAncestor.FocusIndex;

      if (shouldAdjustSize)
      {
        Bus.Invoke(new DetachAndResizeContainerCommand(containerToMove));
        Bus.Invoke(new AttachAndResizeContainerCommand(containerToMove, targetParent, targetIndex));
      }
      else
      {
        Bus.Invoke(new DetachContainerCommand(containerToMove));
        Bus.Invoke(new AttachContainerCommand(containerToMove, targetParent, targetIndex));
      }

      // Set `containerToMove` as focused descendant within target subtree if its original subtree
      // had focus more recently (even if the container is not the last focused within that subtree).
      if (isSubtreeFocused)
        Bus.Invoke(new SetFocusedDescendantCommand(containerToMove, targetParentAncestor));

      // If the focused descendant is moved to the targets subtree, then the target's ancestor
      // should be placed before the original ancestor in LCA's `ChildFocusOrder`.
      if (isFocusedDescendant && isSubtreeFocused)
        lowestCommonAncestor.ChildFocusOrder.ShiftToIndex(
          originalFocusIndex,
          targetParentAncestor
        );

      return CommandResponse.Ok;
    }

    private static void MoveToLowestCommonAncestor(Container containerToMove, Container lowestCommonAncestor, int targetIndex, bool shouldAdjustSize)
    {
      // Keep reference to focus index of container's ancestor in LCA's `ChildFocusOrder`.
      var originalFocusIndex = containerToMove.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor)
        .FocusIndex;

      // Keep reference to container index and number of children that LCA has.
      var originalIndex = containerToMove.Index;
      var originalLcaChildCount = lowestCommonAncestor.Children.Count;

      if (shouldAdjustSize)
        Bus.Invoke(new DetachAndResizeContainerCommand(containerToMove));
      else
        Bus.Invoke(new DetachContainerCommand(containerToMove));

      var newLcaChildCount = lowestCommonAncestor.Children.Count;
      var shouldAdjustTargetIndex = originalLcaChildCount > newLcaChildCount
        && originalIndex < targetIndex;

      // Adjust for when target index changes on detach of container. For example, when shifting a
      // top-level container to the right in a workspace.
      var adjustedTargetIndex = shouldAdjustTargetIndex ? targetIndex - 1 : targetIndex;

      if (shouldAdjustSize)
        Bus.Invoke(new AttachAndResizeContainerCommand(containerToMove, lowestCommonAncestor, adjustedTargetIndex));
      else
        Bus.Invoke(new AttachContainerCommand(containerToMove, lowestCommonAncestor, adjustedTargetIndex));

      lowestCommonAncestor.ChildFocusOrder.ShiftToIndex(originalFocusIndex, containerToMove);
    }
  }
}
