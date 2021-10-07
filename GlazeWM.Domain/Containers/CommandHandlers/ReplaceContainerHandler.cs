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
      var parentContainer = command.ParentContainer;
      var replacementContainers = command.ReplacementContainers;
      var childIndex = command.ChildIndex;

      // TODO: Consider detaching `ReplacementContainer` if it already has a parent.

      var containerToReplace = parentContainer.Children[childIndex];

      foreach (var replacementContainer in replacementContainers)
      {
        replacementContainer.Parent = parentContainer;
        (replacementContainer as IResizable).SizePercentage =
          (containerToReplace as IResizable).SizePercentage * (replacementContainer as IResizable).SizePercentage;
      }

      // Replace the container at the given index.
      var index = parentContainer.Children.IndexOf(containerToReplace);
      parentContainer.Children.InsertRange(index, replacementContainers);
      parentContainer.RemoveChild(containerToReplace);

      // Correct any focus order references to the replaced container.
      parentContainer.ChildFocusOrder.Replace(
        containerToReplace, containerToReplace.LastFocusedChild ?? replacementContainers[0]
      );

      _containerService.SplitContainersToRedraw.Add(parentContainer as SplitContainer);

      return CommandResponse.Ok;
    }
  }
}
