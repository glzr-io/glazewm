using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
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
