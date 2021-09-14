using System;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Infrastructure.WindowsApi.Events
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
