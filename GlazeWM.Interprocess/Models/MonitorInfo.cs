using GlazeWM.Domain.Monitors;

namespace GlazeWM.Interprocess.Models
{
  public sealed class MonitorInfo
  {
    public string Id { get; }

    public string DeviceName { get; }

    public MonitorInfo(Monitor monitor)
    {
      Id = monitor.Id;
      DeviceName = monitor.DeviceName;
    }
  }
}
