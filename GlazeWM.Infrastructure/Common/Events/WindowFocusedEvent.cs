using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowFocusedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowFocusedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
