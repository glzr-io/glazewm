using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;

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
