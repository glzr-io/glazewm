using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private readonly ContainerService _containerService;

    public AttachContainerHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(AttachContainerCommand command)
    {
      var childToAdd = command.ChildToAdd;
      var targetParent = command.TargetParent;
      var targetIndex = command.TargetIndex;

      if (childToAdd.Parent != null)
        throw new Exception("Cannot attach an already attached container. This is a bug.");

      childToAdd.Parent = targetParent;
      targetParent.Children.Insert(targetIndex, childToAdd);
      targetParent.ChildFocusOrder.Add(childToAdd);

      _containerService.ContainersToRedraw.Add(targetParent);

      return CommandResponse.Ok;
    }
  }
}
