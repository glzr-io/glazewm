using LarsWM.Domain.Containers.Commands;
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

    public CommandResponse Handle(AttachContainerCommand command)
    {
      var parent = command.Parent;
      var childToAdd = command.ChildToAdd;

      if (childToAdd.Parent != null)
        _bus.Invoke(new DetachContainerCommand(childToAdd.Parent as SplitContainer, childToAdd));

      parent.Children.Insert(command.InsertPosition, childToAdd);
      childToAdd.Parent = parent;

      // Adjust SizePercentage of self and siblings.
      double defaultPercent = 1.0 / parent.Children.Count;
      foreach (var child in parent.Children)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent);

      // Adjust focus order of ancestors if the attached container is focused.
      if (_containerService.FocusedContainer == childToAdd)
        _bus.Invoke(new SetFocusedDescendantCommand(childToAdd));

      return CommandResponse.Ok;
    }
  }
}
