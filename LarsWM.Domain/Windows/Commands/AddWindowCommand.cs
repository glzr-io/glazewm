using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Windows.Commands
{
    public class AddWindowCommand : Command
    {
        public IntPtr WindowHandle { get; }

        public AddWindowCommand(IntPtr windowHandle)
        {
            WindowHandle = windowHandle;
        }
    }
}
