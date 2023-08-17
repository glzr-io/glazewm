using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowTitleChangedEvent(IntPtr WindowHandle)
    : Event(InfraEvent.WindowTitleChangedEvent)
}
