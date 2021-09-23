using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.WindowsApi.Events
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
