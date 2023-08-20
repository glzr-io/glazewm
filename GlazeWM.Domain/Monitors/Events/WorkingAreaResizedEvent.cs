using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public record WorkingAreaResizedEvent(Monitor AffectedMonitor)
    : Event(DomainEvent.WorkingAreaResized);
}
