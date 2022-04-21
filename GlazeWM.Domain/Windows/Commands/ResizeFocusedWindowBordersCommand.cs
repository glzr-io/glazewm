using GlazeWM.Infrastructure.Bussing;
using System;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowBordersCommand : Command
  {
    public IntPtr WindowHandle { get; }

    public ResizeFocusedWindowBordersCommand(IntPtr windowHandle)
    {
      WindowHandle = windowHandle;
    }
  }
}
