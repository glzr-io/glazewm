using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public class MonitorRemovedEvent : Event
  {
    public string RemovedDeviceName { get; }

    public MonitorRemovedEvent(string removedDeviceName)
    {
      RemovedDeviceName = removedDeviceName;
    }
  }
}
