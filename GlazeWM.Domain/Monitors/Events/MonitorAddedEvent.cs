using GlazeWM.Infrastructure.Bussing;
using System;

namespace GlazeWM.Domain.Monitors.Events
{
  public class MonitorAddedEvent : Event
  {
    public Monitor AddedMonitor { get; }

    public MonitorAddedEvent(Monitor addedMonitor)
    {
      AddedMonitor = addedMonitor;
    }
  }
}
