using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.WindowsApi.Events
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
