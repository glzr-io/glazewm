using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Events
{
  public record WindowManagedEvent(Window ManagedWindow)
    : Event(DomainEvent.WindowManaged);
}
