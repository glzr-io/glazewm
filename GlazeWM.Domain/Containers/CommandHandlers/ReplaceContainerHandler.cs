using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class ReplaceContainerHandler : ICommandHandler<ReplaceContainerCommand>
  {
    private ContainerService _containerService;

    public ReplaceContainerHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(ReplaceContainerCommand command)
    {
      var replacementContainer = command.ReplacementContainer;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      if (replacementContainer.Parent != null)
        throw new Exception(
          "Cannot use an already attached container as replacement container. This is a bug."
        );

      var containerToReplace = targetParent.Children[targetIndex];

      if (containerToReplace is IResizable && replacementContainer is IResizable)
        (replacementContainer as IResizable).SizePercentage =
          (containerToReplace as IResizable).SizePercentage;

      // Adjust `SizePercentage` of siblings.
      if (containerToReplace is IResizable && !(replacementContainer is IResizable))
      {
        // Get the freed up space after container is detached.
        var availableSizePercentage = (containerToReplace as IResizable).SizePercentage;

        var resizableSiblings = containerToReplace.Siblings
          .Where(container => container is IResizable);

        var sizePercentageIncrement = availableSizePercentage / resizableSiblings.Count();

        // Adjust `SizePercentage` of the siblings of the removed container.
        foreach (var sibling in resizableSiblings)
          (sibling as IResizable).SizePercentage =
            (sibling as IResizable).SizePercentage + sizePercentageIncrement;
      }

      // Replace the container at the given index.
      targetParent.Children.Replace(containerToReplace, replacementContainer);
      replacementContainer.Parent = targetParent;
      targetParent.ChildFocusOrder.Replace(containerToReplace, replacementContainer);

      _containerService.ContainersToRedraw.Add(targetParent);

      return CommandResponse.Ok;
    }
  }
}
