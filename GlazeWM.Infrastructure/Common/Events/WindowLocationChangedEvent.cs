using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowLocationChangedEvent(IntPtr WindowHandle)
    : Event(InfraEvent.WindowLocationChanged);
}
