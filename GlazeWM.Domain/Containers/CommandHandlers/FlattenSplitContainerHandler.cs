using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class FlattenSplitContainerHandler : ICommandHandler<FlattenSplitContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public FlattenSplitContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(FlattenSplitContainerCommand command)
    {
      var containerToFlatten = command.ContainerToFlatten;
      var parent = containerToFlatten.Parent;
      var children = containerToFlatten.Children;

      foreach (var child in children)
      {
        child.Parent = parent;
        (child as IResizable).SizePercentage = (containerToFlatten as IResizable).SizePercentage
          * (child as IResizable).SizePercentage;
      }

      // Replace the container at the given index.
      parent.Children.InsertRange(containerToFlatten.Index, children);
      parent.RemoveChild(containerToFlatten);

      // Correct any focus order references to the replaced container.
      var focusIndex = parent.ChildFocusOrder.IndexOf(containerToFlatten);
      parent.ChildFocusOrder.InsertRange(focusIndex, containerToFlatten.ChildFocusOrder);
      parent.ChildFocusOrder.Remove(containerToFlatten);

      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
