using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Events
{
  public class FocusedWindowMovedEvent : Event
  {
    public Window FocusedWindow { get; }
    public SplitContainer OriginalParent { get; }

    public FocusedWindowMovedEvent(Window focusedWindow, SplitContainer originalParent)
    {
      FocusedWindow = focusedWindow;
      OriginalParent = originalParent;
    }
  }
}
