using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.WindowsApi.Events
{
  public class WindowMinimizedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowMinimizedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
