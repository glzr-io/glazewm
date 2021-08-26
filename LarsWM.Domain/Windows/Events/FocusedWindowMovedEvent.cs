using LarsWM.Domain.Containers;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Events
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
