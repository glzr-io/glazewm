using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.Utils;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class SetFocusedDescendantHandler : ICommandHandler<SetFocusedDescendantCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public SetFocusedDescendantHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public dynamic Handle(SetFocusedDescendantCommand command)
    {
      var focusDescendant = command.FocusedDescendant;

      // TODO: This should be moved to somewhere else.
      _containerService.FocusedContainer = focusDescendant;

      // Traverse upwards, setting the container as the last focused.
      var target = focusDescendant;
      while (target.Parent != null)
      {
        target.Parent.ChildFocusOrder.MoveToFront(target);
        target = target.Parent;
      }

      // TODO: This should be moved to somewhere else.
      _bus.RaiseEvent(new FocusChangedEvent(focusDescendant));

      return CommandResponse.Ok;
    }
  }
}
