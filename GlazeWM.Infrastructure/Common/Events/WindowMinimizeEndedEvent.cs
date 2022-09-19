using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowMinimizeEndedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowMinimizeEndedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
