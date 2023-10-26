using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public AttachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(AttachContainerCommand command)
    {
      var childToAdd = command.ChildToAdd;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      if (!childToAdd.IsDetached())
        throw new Exception("Cannot attach an already attached container. This is a bug.");

      targetParent.InsertChild(targetIndex, childToAdd);

      // Return early if child doesn't have to be resized.
      if (childToAdd is not IResizable)
        return CommandResponse.Ok;

      var resizableSiblings = childToAdd.SiblingsOfType<IResizable>();

      if (!resizableSiblings.Any())
      {
        (childToAdd as IResizable).SizePercentage = 1;
        return CommandResponse.Ok;
      }

      var defaultPercent = 1.0 / (resizableSiblings.Count() + 1);

      // Set initial size percentage to 0, and then size up the container to `defaultPercent`.
      (childToAdd as IResizable).SizePercentage = 0;
      _bus.Invoke(new ResizeContainerCommand(childToAdd, defaultPercent));

      _containerService.ContainersToRedraw.Add(targetParent);

      return CommandResponse.Ok;
    }
  }
}
