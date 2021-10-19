using System;
using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class MoveContainerWithinTreeHandler : ICommandHandler<MoveContainerWithinTreeCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public MoveContainerWithinTreeHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(MoveContainerWithinTreeCommand command)
    {
      var container = command.Container;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;
      var insertionPosition = command.InsertionPosition;

      // TODO: Handle case where target parent doesn't have children.
      // TODO: Throw error if `target` is null.

      // Get lowest common ancestor (LCA) between `container` and `target`.
      var target = targetParent.Children[targetIndex];
      var lowestCommonAncestor = _containerService.GetLowestCommonAncestor(container, target);

      // Get ancestors of `container` and `target` that are direct children of the LCA. This could
      // be the `container` or `target` itself if they are direct children of the LCA.
      var containerAncestor = container.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);
      var targetAncestor = target.SelfAndAncestors
        .First(ancestor => ancestor.Parent == lowestCommonAncestor);

      if (containerAncestor == targetAncestor)
        throw new Exception("Container ancestor is the same as target ancestor. This is a bug.");

      // Get whether the container is the focused descendant in its original subtree.
      var isFocusedDescendant = container == containerAncestor
        ? true : containerAncestor.LastFocusedDescendant == container;

      // Get whether the ancestor of `container` appears before `target`'s ancestor in the
      // `ChildFocusOrder` of LCA.
      var originalFocusIndex = containerAncestor.FocusIndex;
      var shouldFocusBefore = originalFocusIndex < targetAncestor.FocusIndex;

      _bus.Invoke(new DetachContainerCommand(container));

      var index = insertionPosition == InsertionPosition.BEFORE ? target.Index : target.Index + 1;
      _bus.Invoke(new AttachContainerCommand(target.Parent as SplitContainer, container, index));

      // Set `container` as focus descendant within target subtree if its original subtree had focus
      // more recently (even if the container is not the last focused within that subtree).
      if (shouldFocusBefore)
        _bus.Invoke(new SetFocusedDescendantCommand(container, targetAncestor));

      // If the focused descendant is moved to the targets subtree, then the target's ancestor
      // should be placed before the original ancestor in LCA's `ChildFocusOrder`.
      if (isFocusedDescendant && shouldFocusBefore)
        lowestCommonAncestor.ChildFocusOrder.ShiftToIndex(
          originalFocusIndex,
          targetAncestor
        );

      return CommandResponse.Ok;
    }
  }
}
