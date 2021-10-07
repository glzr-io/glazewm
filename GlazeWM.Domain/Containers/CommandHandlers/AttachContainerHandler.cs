using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
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

      // Get whether the container has focus prior to making any state changes.
      var isFocusedContainer = _containerService.FocusedContainer == childToAdd;

      if (childToAdd.Parent != null)
        _bus.Invoke(new DetachContainerCommand(childToAdd));

      parent.Children.Insert(command.InsertPosition, childToAdd);
      childToAdd.Parent = parent;

      if (childToAdd is TilingWindow || childToAdd is SplitContainer)
        AdjustSiblingSizes(childToAdd);

      _containerService.ContainersToRedraw.Add(parent);

      // Adjust focus order of ancestors if the attached container is focused.
      if (isFocusedContainer)
        _bus.Invoke(new SetFocusedDescendantCommand(childToAdd));

      return CommandResponse.Ok;
    }

    private void AdjustSiblingSizes(Container childToAdd)
    {
      var resizableSiblings = childToAdd.SelfAndSiblings.Where(container => container is IResizable);
      double defaultPercent = 1.0 / resizableSiblings.Count();

      // Adjust `SizePercentage` of the added container and its siblings.
      foreach (var sibling in resizableSiblings)
        (sibling as IResizable).SizePercentage = defaultPercent;
    }
  }
}
