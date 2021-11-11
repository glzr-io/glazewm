using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class AttachAndResizeContainerHandler : ICommandHandler<AttachAndResizeContainerCommand>
  {
    private Bus _bus;

    public AttachAndResizeContainerHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(AttachAndResizeContainerCommand command)
    {
      var childToAdd = command.ChildToAdd;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      if (!(childToAdd is IResizable))
        throw new Exception("Cannot resize a non-resizable container. This is a bug.");

      _bus.Invoke(new AttachContainerCommand(childToAdd, targetParent, targetIndex));

      var resizableSiblings = childToAdd.Siblings.Where(container => container is IResizable);

      double defaultPercent = 1.0 / (resizableSiblings.Count() + 1);
      (childToAdd as IResizable).SizePercentage = defaultPercent;

      var sizePercentageDecrement = defaultPercent / resizableSiblings.Count();

      // Adjust `SizePercentage` of the added container's siblings.
      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage =
          (sibling as IResizable).SizePercentage - sizePercentageDecrement;

      return CommandResponse.Ok;
    }
  }
}
