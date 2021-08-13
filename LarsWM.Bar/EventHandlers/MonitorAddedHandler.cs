using LarsWM.Bar.Commands;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Bar.EventHandlers
{
  class MonitorAddedHandler : IEventHandler<MonitorAddedEvent>
  {
    private Bus _bus;

    public MonitorAddedHandler(Bus bus)
    {
      _bus = bus;
    }

    public void Handle(MonitorAddedEvent @event)
    {
      _bus.Invoke(new LaunchBarOnMonitorCommand(@event.AddedMonitor));
    }
  }
}
