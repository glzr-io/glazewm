using LarsWM.Domain.Containers;
using LarsWM.Infrastructure.Bussing;
using System;

namespace LarsWM.Domain.Windows.Commands
{
  public class AddWindowCommand : Command
  {
    public IntPtr WindowHandle { get; }
    public SplitContainer TargetParent { get; }
    public bool ShouldRedraw { get; }

    public AddWindowCommand(IntPtr windowHandle, SplitContainer targetParent = null, bool shouldRedraw = true)
    {
      WindowHandle = windowHandle;
      TargetParent = targetParent;
      ShouldRedraw = shouldRedraw;
    }
  }
}
