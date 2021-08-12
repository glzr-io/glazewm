using LarsWM.Domain.Monitors;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Bar.Commands
{
  class LaunchBarOnMonitorCommand : Command
  {
    public Monitor Monitor { get; }

    public LaunchBarOnMonitorCommand(Monitor monitor)
    {
      Monitor = monitor;
    }
  }
}
