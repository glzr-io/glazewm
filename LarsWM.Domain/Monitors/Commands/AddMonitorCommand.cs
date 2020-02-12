using LarsWM.Domain.Common.Models;
using System;
using System.Collections.Generic;
using System.Text;
using System.Windows.Forms;

namespace LarsWM.Domain.Monitors.Commands
{
    class AddMonitorCommand : Command
    {
        public Screen Screen { get; set; }

        public AddMonitorCommand(Screen screen)
        {
            Screen = screen;
        }
    }
}
