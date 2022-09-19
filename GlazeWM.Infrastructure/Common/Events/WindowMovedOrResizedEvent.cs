using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowMovedOrResizedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowMovedOrResizedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
