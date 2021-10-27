using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class ReplaceContainerHandler : ICommandHandler<ReplaceContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public ReplaceContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ReplaceContainerCommand command)
    {
      var parentContainer = command.ParentContainer;
      var replacementContainer = command.ReplacementContainer;
      var childIndex = command.ChildIndex;

      // Detach `ReplacementContainer` if it already has a parent.
      if (replacementContainer.Parent != null)
        _bus.Invoke(new DetachContainerCommand(replacementContainer));

      var containerToReplace = parentContainer.Children[childIndex];

      // Replace the container at the given index.
      parentContainer.Children.Replace(containerToReplace, replacementContainer);
      replacementContainer.Parent = parentContainer;

      if (replacementContainer is IResizable && containerToReplace is IResizable)
        (replacementContainer as IResizable).SizePercentage =
          (containerToReplace as IResizable).SizePercentage;

      // Correct any focus order references to the replaced container.
      parentContainer.ChildFocusOrder.Replace(containerToReplace, replacementContainer);

      _containerService.ContainersToRedraw.Add(parentContainer);
      _containerService.ContainersToRedraw.Add(replacementContainer.Parent);

      return CommandResponse.Ok;
    }
  }
}
