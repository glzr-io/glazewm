using LarsWM.Infrastructure.Bussing;
using System.Windows.Forms;

namespace LarsWM.Domain.Monitors.Commands
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
