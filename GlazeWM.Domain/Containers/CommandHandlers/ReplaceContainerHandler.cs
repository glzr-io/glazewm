using System;
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

      // Replace the container at the given index.
      targetParent.Children.Replace(containerToReplace, replacementContainer);
      replacementContainer.Parent = targetParent;

      if (replacementContainer is IResizable && containerToReplace is IResizable)
        (replacementContainer as IResizable).SizePercentage =
          (containerToReplace as IResizable).SizePercentage;

      // Correct any focus order references to the replaced container.
      targetParent.ChildFocusOrder.Replace(containerToReplace, replacementContainer);

      _containerService.ContainersToRedraw.Add(targetParent);

      return CommandResponse.Ok;
    }
  }
}
