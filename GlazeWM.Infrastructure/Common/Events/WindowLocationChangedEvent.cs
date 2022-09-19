using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowLocationChangedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowLocationChangedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
