using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowMovedOrResizedEvent(IntPtr WindowHandle)
    : Event(InfraEvent.WindowMovedOrResizedEvent)
}
