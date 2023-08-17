using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowShownEvent(IntPtr WindowHandle) : Event(InfraEvent.WindowShownEvent)
}
