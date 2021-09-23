using GlazeWM.Infrastructure.Bussing;
using System.Windows.Forms;

namespace GlazeWM.Domain.Monitors.Commands
{
  public class AddMonitorCommand : Command
  {
    public Screen Screen { get; set; }

    public AddMonitorCommand(Screen screen)
    {
      Screen = screen;
    }
  }
}
