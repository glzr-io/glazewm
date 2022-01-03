using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Commands
{
  public class RemoveMonitorCommand : Command
  {
    public Monitor MonitorToRemove { get; set; }

    public RemoveMonitorCommand(Monitor monitorToRemove)
    {
      MonitorToRemove = monitorToRemove;
    }
  }
}
