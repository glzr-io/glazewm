using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public record TilingDirectionChangedEvent(TilingDirection NewTilingDirection)
    : Event(DomainEvent.TilingDirectionChanged);
}
