using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class RunWindowRulesCommand : Command
  {
    public Window Window { get; }

    public RunWindowRulesCommand(Window window)
    {
      Window = window;
    }
  }
}
