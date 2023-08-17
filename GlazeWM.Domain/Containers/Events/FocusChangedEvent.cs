using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public record FocusChangedEvent(Container FocusedContainer)
    : Event(DomainEvent.FocusChanged);
}
