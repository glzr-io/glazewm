using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Events
{
  public record BindingModeChangedEvent(string BindingMode)
    : Event(DomainEvent.BindingModeChanged);
}
