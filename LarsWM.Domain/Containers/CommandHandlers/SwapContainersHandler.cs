using LarsWM.Domain.Containers.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class SwapContainersHandler : ICommandHandler<SwapContainersCommand>
  {
    private ContainerService _containerService;

    public SwapContainersHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public dynamic Handle(SwapContainersCommand command)
    {
      var firstContainer = command.FirstContainer;
      var secondContainer = command.SecondContainer;

      // Keep references to the original parents.
      var firstParent = firstContainer.Parent;
      var secondParent = secondContainer.Parent;

      var firstContainerIndex = firstContainer.SelfAndSiblings.IndexOf(firstContainer);
      var secondContainerIndex = secondContainer.SelfAndSiblings.IndexOf(secondContainer);

      // Swap the containers. Note that `SizePercentage` is not swapped between the containers.
      firstContainer.Parent.Children[firstContainerIndex] = secondContainer;
      firstContainer.Parent = secondParent;
      secondContainer.Parent.Children[secondContainerIndex] = firstContainer;
      secondContainer.Parent = firstParent;

      // Correct focus stack references to avoid parent containers from referencing containers that
      // aren't children. No need to correct references if the containers have the same parent.
      var isSameParent = firstParent == secondParent;

      if (!isSameParent && firstParent.LastFocusedContainer == firstContainer)
        firstParent.LastFocusedContainer = secondContainer;

      if (!isSameParent && secondParent.LastFocusedContainer == secondContainer)
        secondParent.LastFocusedContainer = firstContainer;

      _containerService.SplitContainersToRedraw.Add(firstContainer.Parent as SplitContainer);
      _containerService.SplitContainersToRedraw.Add(secondContainer.Parent as SplitContainer);

      return CommandResponse.Ok;
    }
  }
}
