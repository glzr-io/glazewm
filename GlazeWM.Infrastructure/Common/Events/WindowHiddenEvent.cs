using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record WindowHiddenEvent(IntPtr WindowHandle) : Event(InfraEvent.WindowHidden);
}
