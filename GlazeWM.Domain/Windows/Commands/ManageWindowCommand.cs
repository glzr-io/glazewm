using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ManageWindowCommand : Command
  {
    public IntPtr WindowHandle { get; }
    public SplitContainer TargetParent { get; }

    public ManageWindowCommand(
      IntPtr windowHandle,
      SplitContainer targetParent = null)
    {
      WindowHandle = windowHandle;
      TargetParent = targetParent;
    }
  }
}
