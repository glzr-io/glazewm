using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.WindowsApi.Events
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
