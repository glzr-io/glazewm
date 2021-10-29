using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public DetachContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DetachContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;

      childToRemove.Parent = null;
      parent.Children.Remove(childToRemove);
      parent.ChildFocusOrder.Remove(childToRemove);

      var isEmptySplitContainer = parent is SplitContainer && !parent.HasChildren()
        && !(parent is Workspace);

      // If the parent of the removed child is an empty split container, detach the split container
      // as well.
      if (isEmptySplitContainer)
      {
        _bus.Invoke(new DetachContainerCommand(parent));
        return CommandResponse.Ok;
      }

      return CommandResponse.Ok;
    }
  }
}
