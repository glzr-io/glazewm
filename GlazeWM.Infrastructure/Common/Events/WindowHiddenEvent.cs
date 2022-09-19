using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowHiddenEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowHiddenEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
