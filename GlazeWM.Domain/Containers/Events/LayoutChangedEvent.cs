using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public class LayoutChangedEvent : Event
  {
    public Layout NewLayout { get; }

    public LayoutChangedEvent(Layout newLayout)
    {
      NewLayout = newLayout;
    }
  }
}
