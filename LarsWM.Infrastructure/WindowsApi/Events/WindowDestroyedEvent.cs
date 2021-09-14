using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
{
  public class WindowDestroyedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowDestroyedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
