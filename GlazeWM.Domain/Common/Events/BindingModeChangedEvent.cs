using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Events
{
  public record BindingModeChangedEvent(string NewBindingMode)
    : Event(DomainEvent.BindingModeChanged);
}
