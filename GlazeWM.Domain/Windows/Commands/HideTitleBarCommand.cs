using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class HideTitleBarCommand : Command
  {
    public Window TargetWindow { get; }

    public HideTitleBarCommand(Window targetWindow)
    {
      TargetWindow = targetWindow;
    }
  }
}
