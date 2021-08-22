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
      var containerToReplace = command.ContainerToReplace;
      var replacementContainer = command.ReplacementContainer;
      var targetIndex = containerToReplace.SelfAndSiblings.IndexOf(containerToReplace);

      // Replace the split container with the focused window.
      containerToReplace.Parent.Children[targetIndex] = replacementContainer;
      replacementContainer.Parent = containerToReplace.Parent;
      replacementContainer.SizePercentage = containerToReplace.SizePercentage;

      // Correct focus stack references to replaced container.
      if (containerToReplace.Parent.LastFocusedContainer == containerToReplace)
        containerToReplace.Parent.LastFocusedContainer = replacementContainer;

      // TODO: Not sure whether redrawing is necessary, will see after fixing detach command.
      _containerService.SplitContainersToRedraw.Add(containerToReplace.Parent as SplitContainer);
      _containerService.SplitContainersToRedraw.Add(replacementContainer.Parent as SplitContainer);

      return CommandResponse.Ok;
    }
  }
}
