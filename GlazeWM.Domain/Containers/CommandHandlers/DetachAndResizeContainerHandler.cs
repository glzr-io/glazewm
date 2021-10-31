using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class DetachAndResizeContainerHandler : ICommandHandler<DetachAndResizeContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public DetachAndResizeContainerHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(DetachAndResizeContainerCommand command)
    {
      var childToRemove = command.ChildToRemove;
      var parent = childToRemove.Parent;
      var grandparent = parent.Parent;

      if (!(childToRemove is TilingWindow || childToRemove is SplitContainer))
        return CommandResponse.Ok;

      _bus.Invoke(new DetachContainerCommand(childToRemove));

      // TODO: Adjust `SizePercentage` of children based on their previous `SizePercentage`.

      // Resize children of grandparent if `childToRemove`'s parent was also detached (ie. in the
      // case of an empty split container).
      var isParentDetached = parent.Parent == null;
      var containersToResize = isParentDetached
        ? grandparent.Children.Where(container => container is IResizable)
        : parent.Children.Where(container => container is IResizable);

      double defaultPercent = 1.0 / containersToResize.Count();

      // Adjust `SizePercentage` of the siblings of the removed container.
      foreach (var sibling in containersToResize)
        (sibling as IResizable).SizePercentage = defaultPercent;

      _containerService.ContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
