using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public class TilingDirectionChangedEvent : Event
  {
    public TilingDirection NewTilingDirection { get; }

    public TilingDirectionChangedEvent(TilingDirection newTilingDirection)
    {
      NewTilingDirection = newTilingDirection;
    }
  }
}
