using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public DetachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DetachContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;

      if (parent == null)
        throw new Exception("Cannot detach an already detached container. This is a bug.");

      childToRemove.Parent = null;
      parent.Children.Remove(childToRemove);
      parent.ChildFocusOrder.Remove(childToRemove);

      var isSplitContainer = parent is SplitContainer and not Workspace;
      if (isSplitContainer)
      {
        // If the parent of the removed child is an empty split container, detach the split container
        // as well.
        if (!parent.HasChildren())
        {
          _bus.Invoke(new DetachContainerCommand(parent));
          return CommandResponse.Ok;
        }

        // If the parent of the removed child is a split container with an only child that is itself a split container,
        // flatten the outer, unnecessary split container
        if (parent.Children.Count == 1 && parent.Children[0] is SplitContainer)
        {
          (parent.Children[0] as IResizable).SizePercentage = 1; // The only child now takes up the full container
          _bus.Invoke(new FlattenSplitContainerCommand(parent as SplitContainer));
          return CommandResponse.Ok;
        }
      }

      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
