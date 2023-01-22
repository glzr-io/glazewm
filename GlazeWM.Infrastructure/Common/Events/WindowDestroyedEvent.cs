using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowDestroyedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowDestroyedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
