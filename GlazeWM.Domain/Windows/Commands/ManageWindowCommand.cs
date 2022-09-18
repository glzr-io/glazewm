using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Bussing;
using System;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ManageWindowCommand : Command
  {
    public IntPtr WindowHandle { get; }
    public SplitContainer TargetParent { get; }
    public bool ShouldRedraw { get; }

    public ManageWindowCommand(
      IntPtr windowHandle,
      SplitContainer targetParent = null,
      bool shouldRedraw = true)
    {
      WindowHandle = windowHandle;
      TargetParent = targetParent;
      ShouldRedraw = shouldRedraw;
    }
  }
}
