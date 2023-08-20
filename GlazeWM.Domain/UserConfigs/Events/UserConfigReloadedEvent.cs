using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.Events
{
  public record UserConfigReloadedEvent() : Event(DomainEvent.UserConfigReloaded);
}
