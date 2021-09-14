using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
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
