using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class ChangeFocusedContainerLayoutHandler : ICommandHandler<ChangeFocusedContainerLayoutCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ChangeFocusedContainerLayoutHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ChangeFocusedContainerLayoutCommand command)
    {
      if (!_containerService.IsFocusSynced)
        return CommandResponse.Ok;

      var focusedContainer = _containerService.FocusedContainer;
      Bus.Invoke(new ChangeContainerLayoutCommand(focusedContainer, command.NewLayout));

      return CommandResponse.Ok;
    }
  }
}
