using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public record NativeFocusReassignedEvent(Container FocusedContainer)
    : Event(DomainEvent.FocusChanged);
}
