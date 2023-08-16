using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Events
{
  public class WindowManagedEvent : Event
  {
    public Window Window { get; }

    public WindowManagedEvent(Window window)
    {
      Window = window;
    }
  }
}
