using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class UnmanageWindowCommand : Command
  {
    public Window Window { get; }

    public UnmanageWindowCommand(Window window)
    {
      Window = window;
    }
  }
}
