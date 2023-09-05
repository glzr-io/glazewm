using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public record FocusedContainerMovedEvent(Container FocusedContainer)
    : Event(DomainEvent.FocusedContainerMoved);
}
