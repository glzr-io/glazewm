using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public record MonitorRemovedEvent(string RemovedDeviceName)
    : Event(DomainEvent.MonitorRemoved);
}
