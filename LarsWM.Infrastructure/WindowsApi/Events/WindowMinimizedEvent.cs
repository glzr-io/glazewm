using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
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
