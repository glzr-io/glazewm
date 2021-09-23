using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Events
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
