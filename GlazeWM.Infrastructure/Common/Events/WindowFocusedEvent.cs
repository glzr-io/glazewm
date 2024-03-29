using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowFocusedEvent(IntPtr WindowHandle) : Event(InfraEvent.WindowFocused);
}
