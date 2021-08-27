using LarsWM.Domain.Containers.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class ReplaceContainerHandler : ICommandHandler<ReplaceContainerCommand>
  {
    private ContainerService _containerService;

    public ReplaceContainerHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public dynamic Handle(ReplaceContainerCommand command)
    {
      var parentContainer = command.ParentContainer;
      var replacementContainer = command.ReplacementContainer;
      var childIndex = command.ChildIndex;
      var containerToReplace = parentContainer.Children[childIndex];

      // TODO: Consider detaching `ReplacementContainer` if it already has a parent.

      // Replace the container at the given index.
      parentContainer.Children[childIndex] = replacementContainer;
      replacementContainer.Parent = parentContainer;
      replacementContainer.SizePercentage = containerToReplace.SizePercentage;

      // Correct focus stack references to replaced container.
      if (parentContainer.LastFocusedChild == containerToReplace)
        parentContainer.LastFocusedChild = replacementContainer;

      // TODO: Not sure whether redrawing is necessary, will see after fixing detach command.
      _containerService.SplitContainersToRedraw.Add(parentContainer as SplitContainer);
      _containerService.SplitContainersToRedraw.Add(replacementContainer.Parent as SplitContainer);

      return CommandResponse.Ok;
    }
  }
}
