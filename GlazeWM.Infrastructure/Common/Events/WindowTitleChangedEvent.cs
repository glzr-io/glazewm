using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public class WindowTitleChangedEvent : Event
  {
    public IntPtr WindowHandle { get; }

    public WindowTitleChangedEvent(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}