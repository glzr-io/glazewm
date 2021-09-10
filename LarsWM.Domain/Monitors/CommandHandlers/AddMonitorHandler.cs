using LarsWM.Domain.Containers;
using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.CommandHandlers
{
  class AddMonitorHandler : ICommandHandler<AddMonitorCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public AddMonitorHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(AddMonitorCommand command)
    {
      var newMonitor = new Monitor(command.Screen);
      _containerService.ContainerTree.Add(newMonitor);

      _bus.RaiseEvent(new MonitorAddedEvent(newMonitor));

      return CommandResponse.Ok;
    }
  }
}
