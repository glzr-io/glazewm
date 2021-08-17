using LarsWM.Domain.Containers;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Events
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
