using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Monitors.Events
{
    public class MonitorAddedEvent : Event
    {
        public Guid AddedMonitorId { get; }

        public MonitorAddedEvent(Guid addedMonitorId)
        {
            AddedMonitorId = addedMonitorId;
        }
    }
}
