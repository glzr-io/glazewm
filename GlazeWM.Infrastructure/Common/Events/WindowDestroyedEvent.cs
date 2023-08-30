using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowDestroyedEvent(IntPtr WindowHandle)
    : Event(InfraEvent.WindowDestroyed);
}
