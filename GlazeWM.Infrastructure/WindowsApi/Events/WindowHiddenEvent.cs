using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.WindowsApi.Events
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
