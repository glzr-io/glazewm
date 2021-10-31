using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private ContainerService _containerService;

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

      // TODO: Insert at end of parent's `ChildFocusOrder`.
      targetParent.Children.Insert(targetIndex, childToAdd);
      childToAdd.Parent = targetParent;

      _containerService.ContainersToRedraw.Add(targetParent);

      return CommandResponse.Ok;
    }
  }
}
