using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public AttachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(AttachContainerCommand command)
    {
      var parent = command.Parent;
      var childToAdd = command.ChildToAdd;

      if (childToAdd.Parent != null)
        throw new Exception("Attempting to attach an already attached container. This is a bug.");

      // TODO: Insert at end of parent's `ChildFocusOrder`.
      parent.Children.Insert(command.InsertPosition, childToAdd);
      childToAdd.Parent = parent;

      _containerService.ContainersToRedraw.Add(parent);


      return CommandResponse.Ok;
    }
  }
}
