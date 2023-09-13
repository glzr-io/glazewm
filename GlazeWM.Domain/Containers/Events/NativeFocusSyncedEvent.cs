using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public record NativeFocusSyncedEvent(Container FocusedContainer)
    : Event(DomainEvent.NativeFocusSynced);
}
