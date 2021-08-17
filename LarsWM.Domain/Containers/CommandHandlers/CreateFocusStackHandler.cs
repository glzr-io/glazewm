using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Workspaces.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class CreateFocusStackHandler : ICommandHandler<CreateFocusStackCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public CreateFocusStackHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public dynamic Handle(CreateFocusStackCommand command)
    {
      var target = command.FocusedContainer;
      var focusedContainer = _containerService.FocusedContainer;

      if (target == focusedContainer)
        return CommandResponse.Ok;

      _containerService.FocusedContainer = target;

      // Traverse upwards, creating a focus stack from the focused container.
      while (target.Parent != null)
      {
        target.Parent.LastFocusedContainer = target;
        target = target.Parent;
      }

      _bus.RaiseEvent(new FocusChangedEvent(command.FocusedContainer));

      return CommandResponse.Ok;
    }
  }
}
