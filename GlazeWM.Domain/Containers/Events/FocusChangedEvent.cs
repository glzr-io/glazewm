using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
{
  public class FocusChangedEvent : Event
  {
    public Container FocusedContainer { get; }

    public FocusChangedEvent(Container focusedContainer)
    {
      FocusedContainer = focusedContainer;
    }
  }
}
