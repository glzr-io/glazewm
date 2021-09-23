using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Domain.Containers.CommandHandlers
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

    public CommandResponse Handle(SetFocusedDescendantCommand command)
    {
      var focusDescendant = command.FocusedDescendant;

      // Traverse upwards, setting the container as the last focused.
      var target = focusDescendant;
      while (target.Parent != null)
      {
        target.Parent.ChildFocusOrder.MoveToFront(target);
        target = target.Parent;
      }

      return CommandResponse.Ok;
    }
  }
}
