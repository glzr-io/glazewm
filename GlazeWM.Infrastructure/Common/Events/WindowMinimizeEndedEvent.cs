using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowMinimizeEndedEvent(IntPtr WindowHandle)
    : Event(InfraEvent.WindowMinimizeEnded);
}
