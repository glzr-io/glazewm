using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Monitors.Events
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
