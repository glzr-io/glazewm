using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class SetMaximizedCommand : Command
  {
    public Window Window { get; }

    public SetMaximizedCommand(Window window)
    {
      Window = window;
    }
  }
}
