using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class RemoveTitleBarCommand : Command
  {
    public Window Window { get; }

    public RemoveTitleBarCommand(Window window)
    {
      Window = window;
    }
  }
}
