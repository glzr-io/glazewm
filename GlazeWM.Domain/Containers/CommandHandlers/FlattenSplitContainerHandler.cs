using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

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

      // Keep references to properties of container to flatten prior to detaching.
      var originalFocusIndex = containerToFlatten.FocusIndex;
      var originalIndex = containerToFlatten.Index;
      var originalFocusOrder = containerToFlatten.ChildFocusOrder.ToList();

      _bus.Invoke(new DetachContainerCommand(containerToFlatten));

      // Insert children of detached split container at the original index.
      foreach (var (child, index) in children.WithIndex())
      {
        _bus.Invoke(new AttachContainerCommand(parent, child, originalIndex + index));

        (child as IResizable).SizePercentage = (containerToFlatten as IResizable).SizePercentage
          * (child as IResizable).SizePercentage;
      }

      // Correct focus order of the inserted containers.
      foreach (var child in children)
      {
        var childFocusIndex = originalFocusOrder.IndexOf(child);
        parent.ChildFocusOrder.ShiftToIndex(originalFocusIndex + childFocusIndex, child);
      }

      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
