using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public class WorkingAreaResizedEvent : Event
  {
    public Monitor AffectedMonitor { get; }

    public WorkingAreaResizedEvent(Monitor affectedMonitor)
    {
      AffectedMonitor = affectedMonitor;
    }
  }
}
