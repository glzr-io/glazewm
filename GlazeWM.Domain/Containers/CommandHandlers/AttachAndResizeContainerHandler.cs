using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class AttachAndResizeContainerHandler : ICommandHandler<AttachAndResizeContainerCommand>
  {
    private readonly Bus _bus;

    public AttachAndResizeContainerHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(AttachAndResizeContainerCommand command)
    {
      var childToAdd = command.ChildToAdd;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      if (childToAdd is not IResizable)
        throw new Exception("Cannot resize a non-resizable container. This is a bug.");

      _bus.Invoke(new AttachContainerCommand(childToAdd, targetParent, targetIndex));

      var resizableSiblings = childToAdd.SiblingsOfType<IResizable>();
      var defaultPercent = 1.0 / (resizableSiblings.Count() + 1);

      // TODO: If called with a container that has a `SizePercentage` of 1, them it gets increased to 1.5.
      // Could instead have a `SetSizePercentageCommand`.
      _bus.Invoke(new ResizeContainerCommand(childToAdd, defaultPercent));

      return CommandResponse.Ok;
    }
  }
}
