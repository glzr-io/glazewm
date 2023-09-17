using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class RoundWindowBorderCommand: Command
  {
    public Window TargetWindow { get; }

    public RoundWindowBorderCommand(Window targetWindow)
    {
      TargetWindow = targetWindow;
    }
  }
}
