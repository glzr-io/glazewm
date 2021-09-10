using LarsWM.Domain.Containers.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.Utils;

namespace LarsWM.Domain.Containers.CommandHandlers
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
      var replacementContainer = command.ReplacementContainer;
      var childIndex = command.ChildIndex;

      // TODO: Consider detaching `ReplacementContainer` if it already has a parent.

      // Replace the container at the given index.
      var containerToReplace = parentContainer.Children[childIndex];
      parentContainer.Children.Replace(containerToReplace, replacementContainer);
      replacementContainer.Parent = parentContainer;
      replacementContainer.SizePercentage = containerToReplace.SizePercentage;

      // Correct any focus order references to the replaced container.
      parentContainer.ChildFocusOrder.Replace(containerToReplace, replacementContainer);

      _containerService.SplitContainersToRedraw.Add(parentContainer as SplitContainer);
      _containerService.SplitContainersToRedraw.Add(replacementContainer.Parent as SplitContainer);

      return CommandResponse.Ok;
    }
  }
}
