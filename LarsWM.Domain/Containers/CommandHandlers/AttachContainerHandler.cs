using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
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

    public dynamic Handle(AttachContainerCommand command)
    {
      var parent = command.Parent;
      var newChild = command.ChildToAdd;

      if (newChild.Parent != null)
        _bus.Invoke(new DetachContainerCommand(newChild.Parent as SplitContainer, newChild));

      // TODO: Adjust SizePercentage of current children.
      parent.Children.Insert(command.InsertPosition, newChild);
      newChild.Parent = parent;

      double defaultPercent = 1.0 / parent.Children.Count;
      foreach (var child in parent.Children)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
