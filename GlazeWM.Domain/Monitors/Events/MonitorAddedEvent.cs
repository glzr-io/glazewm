using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public record MonitorAddedEvent(Monitor AddedMonitor)
    : Event(DomainEvent.MonitorAdded);
}
