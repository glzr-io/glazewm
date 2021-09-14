using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
{
  public class WindowFocusedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowFocusedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
