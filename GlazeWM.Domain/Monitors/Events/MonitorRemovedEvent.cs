using System;
using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public record MonitorRemovedEvent(Guid RemovedId, string RemovedDeviceName)
    : Event(DomainEvent.MonitorRemoved);
}
