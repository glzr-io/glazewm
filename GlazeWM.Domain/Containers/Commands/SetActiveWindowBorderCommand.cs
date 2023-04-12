using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers.Commands
{
  public class SetActiveWindowBorderCommand : Command
  {
    public Window TargetWindow { get; }

    /// <summary>
    /// Sets the newly focused window's border and removes border on older window.  
    /// </summary>
    public SetActiveWindowBorderCommand(Window target)
    {
      TargetWindow = target;
    }
  }
}
