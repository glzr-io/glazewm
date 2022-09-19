using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowShownEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowShownEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
